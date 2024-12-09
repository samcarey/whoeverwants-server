use crate::command::Command;
use anyhow::{bail, Context, Result};
use axum::{
    response::{Html, IntoResponse},
    routing::post,
    Extension, Form, Router,
};
use contacts::{add_contact, process_contact_submission};
use dotenv::dotenv;
use enum_iterator::all;
use log::*;
use once_cell::sync::Lazy;
use openapi::apis::{
    api20100401_message_api::{create_message, CreateMessageParams},
    configuration::Configuration,
};
use sqlx::{query, query_as, Pool, Sqlite};
use std::env;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::{collections::HashMap, str::FromStr};
use util::E164;

mod command;
mod contacts;
#[cfg(test)]
mod test;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;
    env_logger::init();
    info!("Starting up");
    let twilio_config = Configuration {
        basic_auth: Some((
            env::var("TWILIO_API_KEY_SID")?,
            Some(env::var("TWILIO_API_KEY_SECRET")?),
        )),
        ..Default::default()
    };
    send(
        &twilio_config,
        env::var("CLIENT_NUMBER")?,
        "Server is starting up".to_string(),
    )
    .await?;
    let pool = sqlx::SqlitePool::connect(&env::var("DATABASE_URL")?).await?;
    query!("PRAGMA foreign_keys = ON").execute(&pool).await?; // SQLite has this off by default
    let app = Router::new()
        .route("/", post(handle_incoming_sms))
        .layer(Extension(pool));
    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        env::var("CALLBACK_IP")?,
        env::var("CALLBACK_PORT")?
    ))
    .await?;
    info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

// field names must be exact (including case) to match API
#[allow(non_snake_case)]
#[derive(serde::Deserialize, Default, Debug)]
struct SmsMessage {
    Body: String,
    From: String,
    NumMedia: Option<String>,
    MediaContentType0: Option<String>,
    MediaUrl0: Option<String>,
}

struct User {
    number: String,
    #[allow(dead_code)]
    name: String,
}

#[derive(Clone, sqlx::FromRow)]
struct Contact {
    id: i64,
    contact_name: String,
    contact_user_number: String,
}

#[derive(Debug)]
enum ImportResult {
    Added,
    Updated,
    Unchanged,
    Deferred,
}

// Handler for incoming SMS messages
async fn handle_incoming_sms(
    Extension(pool): Extension<Pool<Sqlite>>,
    Form(message): Form<SmsMessage>,
) -> impl IntoResponse {
    let response = match process_message(&pool, message).await {
        Ok(response) => response,
        Err(error) => {
            error!("Error: {error:?}");
            "Internal Server Error!".to_string()
        }
    };
    debug!("Sending response: {response}");
    Html(format!(
        r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Response>
        <Message>{response}</Message>
        </Response>
        "#
    ))
}

async fn process_message(pool: &Pool<Sqlite>, message: SmsMessage) -> anyhow::Result<String> {
    trace!("Received {message:?}");
    let SmsMessage {
        Body: body,
        From: from,
        NumMedia,
        MediaContentType0,
        MediaUrl0,
    } = message;
    debug!("Received from {from}: {body}");
    if NumMedia == Some("1".to_string())
        && MediaContentType0
            .map(|t| ["text/vcard", "text/x-vcard"].contains(&t.as_str()))
            .unwrap_or(false)
    {
        return process_contact_submission(pool, &from, &MediaUrl0).await;
    }

    let mut words = body.trim().split_ascii_whitespace();
    let command_word = words.next();
    let command = command_word.map(Command::try_from);

    let Some(User {
        number, name: _, ..
    }) = query_as!(User, "select * from users where number = ?", from)
        .fetch_optional(pool)
        .await?
    else {
        return onboard_new_user(command, words, &from, pool).await;
    };

    let Some(command) = command else {
        return Ok(Command::h.hint());
    };

    let Ok(command) = command else {
        return Ok(format!(
            "We didn't recognize that command word: \"{}\".\n{}",
            command_word.unwrap(),
            Command::h.hint()
        ));
    };

    let response = match command {
        // I would use HELP for the help command, but Twilio intercepts and does not relay that
        Command::h => {
            let available_commands = format!(
                "Available commands:\n{}\n",
                all::<Command>()
                    .map(|c| format!("- {c}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            format!("{available_commands}\n{}", Command::info.hint())
        }
        Command::name => match process_name(words) {
            Ok(name) => {
                query!("update users set name = ? where number = ?", name, from)
                    .execute(pool)
                    .await?;
                format!("Your name has been updated to \"{name}\"")
            }
            Err(hint) => hint.to_string(),
        },
        Command::stop => {
            query!("delete from users where number = ?", number)
                .execute(pool)
                .await?;
            // They won't actually see this when using Twilio
            "You've been unsubscribed. Goodbye!".to_string()
        }
        Command::info => {
            let command_text = words.next();
            if let Some(command) = command_text.map(Command::try_from) {
                if let Ok(command) = command {
                    format!(
                        "{}, to {}.{}",
                        command.usage(),
                        command.description(),
                        command.example()
                    )
                } else {
                    format!("Command \"{}\" not recognized", command_text.unwrap())
                }
            } else {
                Command::info.hint()
            }
        }
        Command::contacts => {
            let contacts = query_as!(
                Contact,
                "SELECT id as \"id!\", contact_name, contact_user_number FROM contacts WHERE submitter_number = ? ORDER BY contact_name",
                from
            )
            .fetch_all(pool)
            .await?;

            if contacts.is_empty() {
                "You don't have any contacts.".to_string()
            } else {
                let contact_list = contacts
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        format!(
                            "{}. {} ({})",
                            i + 1,
                            c.contact_name,
                            &E164::from_str(&c.contact_user_number)
                                .expect("Should have been formatted upon db insertion")
                                .area_code()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("Your contacts:\n{}", contact_list)
            }
        }
        Command::delete => {
            let name = words.collect::<Vec<_>>().join(" ");
            if name.is_empty() {
                Command::delete.hint()
            } else {
                handle_delete(pool, &from, &name).await?
            }
        }
        Command::confirm => {
            let nums = words.collect::<Vec<_>>().join(" ");
            if nums.is_empty() {
                Command::confirm.hint()
            } else {
                handle_confirm(pool, &from, &nums).await?
            }
        }
        Command::pick => {
            let nums = words.collect::<Vec<_>>().join(" ");
            if nums.is_empty() {
                Command::pick.hint()
            } else {
                handle_pick(pool, &from, &nums).await?
            }
        }
        Command::group => {
            let names = words.collect::<Vec<_>>().join(" ");
            if names.is_empty() {
                Command::group.hint()
            } else {
                handle_group(pool, &from, &names).await?
            }
        }
    };
    Ok(response)
}

async fn handle_group(pool: &Pool<Sqlite>, from: &str, names: &str) -> anyhow::Result<String> {
    cleanup_pending_deletions();

    let name_fragments: Vec<_> = names.split(',').map(str::trim).collect();

    if name_fragments.is_empty() {
        return Ok("Please provide at least one name to search for.".to_string());
    }

    let mut contacts = Vec::new();
    for fragment in &name_fragments {
        let like = format!("%{}%", fragment.to_lowercase());
        let mut matches = query_as!(
            Contact,
            "SELECT id as \"id!\", contact_name, contact_user_number 
             FROM contacts 
             WHERE submitter_number = ? 
             AND LOWER(contact_name) LIKE ?
             ORDER BY contact_name",
            from,
            like
        )
        .fetch_all(pool)
        .await?;
        contacts.append(&mut matches);
    }

    contacts.sort_by(|a, b| a.id.cmp(&b.id));
    contacts.dedup_by(|a, b| a.id == b.id);
    contacts.sort_by(|a, b| a.contact_name.cmp(&b.contact_name));

    if contacts.is_empty() {
        return Ok(format!(
            "No contacts found matching: {}",
            name_fragments.join(", ")
        ));
    }

    let list = contacts
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let area_code = E164::from_str(&c.contact_user_number)
                .map(|e| e.area_code().to_string())
                .unwrap_or_else(|_| "???".to_string());

            format!("{}. {} ({})", i + 1, c.contact_name, area_code)
        })
        .collect::<Vec<_>>()
        .join("\n");

    for (i, contact) in contacts.iter().enumerate() {
        let token = format!("{}:{}", from, i + 1);
        PENDING_ACTIONS.lock().unwrap().insert(
            token,
            PendingAction {
                contact_id: contact.id,
                timestamp: Instant::now(),
                intent: ConfirmationIntent::AddToGroup,
            },
        );
    }

    Ok(format!(
        "Found these contacts:\n{}\n\nTo create a group with these contacts, reply \"confirm NUM1, NUM2, ...\"",
        list
    ))
}

async fn handle_pick(pool: &Pool<Sqlite>, from: &str, selections: &str) -> anyhow::Result<String> {
    // First check if there are any deferred contacts
    let deferred_contacts = query!(
        "SELECT DISTINCT contact_name FROM deferred_contacts WHERE submitter_number = ?",
        from
    )
    .fetch_all(pool)
    .await?;

    if deferred_contacts.is_empty() {
        return Ok("No pending contacts to pick from.".to_string());
    }

    let mut successful = Vec::new();
    let mut failed = Vec::new();

    // Parse selections like "1a, 2b, 3a"
    for selection in selections.split(',').map(str::trim) {
        if selection.len() < 2 {
            failed.push(format!("Invalid selection format: {}", selection));
            continue;
        }

        // Split into numeric and letter parts
        let (num_str, letter) = selection.split_at(selection.len() - 1);
        let contact_idx: usize = match num_str.parse::<usize>() {
            Ok(n) if n > 0 => n - 1,
            _ => {
                failed.push(format!("Invalid contact number: {}", num_str));
                continue;
            }
        };

        let letter_idx = match letter.chars().next().unwrap() {
            c @ 'a'..='z' => (c as u8 - b'a') as usize,
            _ => {
                failed.push(format!("Invalid letter selection: {}", letter));
                continue;
            }
        };

        // Get the contact name and number
        let Some(contact_name) = deferred_contacts.get(contact_idx).map(|c| &c.contact_name) else {
            failed.push(format!("Contact number {} not found", contact_idx + 1));
            continue;
        };

        // Get all numbers for this contact
        let numbers = query!(
            "SELECT phone_number, phone_description FROM deferred_contacts 
             WHERE submitter_number = ? AND contact_name = ?
             ORDER BY id",
            from,
            contact_name
        )
        .fetch_all(pool)
        .await?;

        let Some(number) = numbers.get(letter_idx) else {
            failed.push(format!(
                "Number {} not found for contact {}",
                letter,
                contact_idx + 1
            ));
            continue;
        };

        // Insert the contact
        if let Err(e) = add_contact(pool, from, contact_name, &number.phone_number).await {
            failed.push(format!(
                "Failed to add {} ({}): {}",
                contact_name, number.phone_number, e
            ));
        } else {
            successful.push(format!("{} ({})", contact_name, number.phone_number));
        }
    }

    // Clean up processed contacts
    for contact in &successful {
        if let Some(name) = contact.split(" (").next() {
            query!(
                "DELETE FROM deferred_contacts WHERE submitter_number = ? AND contact_name = ?",
                from,
                name
            )
            .execute(pool)
            .await?;
        }
    }

    // Format response
    let mut response = String::new();
    if !successful.is_empty() {
        response.push_str(&format!(
            "Successfully added {} contact{}:\n",
            successful.len(),
            if successful.len() == 1 { "" } else { "s" }
        ));
        for contact in successful {
            response.push_str(&format!("• {}\n", contact));
        }
    }

    if !failed.is_empty() {
        if !response.is_empty() {
            response.push_str("\n");
        }
        response.push_str("Failed to process:\n");
        for error in failed {
            response.push_str(&format!("• {}\n", error));
        }
    }

    Ok(response)
}

async fn handle_delete(pool: &Pool<Sqlite>, from: &str, name: &str) -> anyhow::Result<String> {
    cleanup_pending_deletions();

    let like = format!("%{}%", name.to_lowercase());
    let contacts = query_as!(
        Contact,
        "SELECT id as \"id!\", contact_name, contact_user_number 
         FROM contacts 
         WHERE submitter_number = ? 
         AND LOWER(contact_name) LIKE ?
         ORDER BY contact_name",
        from,
        like
    )
    .fetch_all(pool)
    .await?;

    if contacts.is_empty() {
        return Ok(format!("No contacts found matching \"{}\"", name));
    }

    let list = contacts
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let area_code = E164::from_str(&c.contact_user_number)
                .map(|e| e.area_code().to_string())
                .unwrap_or_else(|_| "???".to_string());

            format!("{}. {} ({})", i + 1, c.contact_name, area_code)
        })
        .collect::<Vec<_>>()
        .join("\n");

    for (i, contact) in contacts.iter().enumerate() {
        let token = format!("{}:{}", from, i + 1);
        PENDING_ACTIONS.lock().unwrap().insert(
            token,
            PendingAction {
                contact_id: contact.id,
                timestamp: Instant::now(),
                intent: ConfirmationIntent::Delete,
            },
        );
    }

    Ok(format!(
        "Found these contacts matching \"{}\":\n{}\n\n\
        To delete contacts, reply \"confirm NUM1, NUM2, ...\", \
        where NUM1, NUM2, etc. are numbers from the list above.",
        name, list
    ))
}

async fn handle_confirm(
    pool: &Pool<Sqlite>,
    from: &str,
    selections: &str,
) -> anyhow::Result<String> {
    cleanup_pending_deletions();

    // First, collect all the necessary information while holding the lock
    let (contact_ids, intent, invalid) = {
        let pending = PENDING_ACTIONS.lock().unwrap();
        let mut selected_items = Vec::new();
        let mut invalid = Vec::new();
        let mut intent = None;

        // Process all selections and collect the data
        for num_str in selections.split(',').map(str::trim) {
            match num_str.parse::<usize>() {
                Ok(num) if num > 0 => {
                    let token = format!("{}:{}", from, num);
                    if let Some(deletion) = pending.get(&token) {
                        selected_items.push(deletion.contact_id);
                        if intent.is_none() {
                            intent = Some(match deletion.intent {
                                ConfirmationIntent::Delete => ConfirmationIntent::Delete,
                                ConfirmationIntent::AddToGroup => ConfirmationIntent::AddToGroup,
                            });
                        } else if !matches!(
                            (&deletion.intent, intent.as_ref().unwrap()),
                            (ConfirmationIntent::Delete, ConfirmationIntent::Delete)
                                | (
                                    ConfirmationIntent::AddToGroup,
                                    ConfirmationIntent::AddToGroup
                                )
                        ) {
                            invalid.push(
                                "Cannot mix deletion and group creation selections".to_string(),
                            );
                        }
                    } else {
                        invalid.push(format!("Invalid selection: {}", num));
                    }
                }
                _ => invalid.push(format!("Invalid number: {}", num_str)),
            }
        }

        if selected_items.is_empty() && invalid.is_empty() {
            return Ok("No valid selections provided.".to_string());
        }

        (
            selected_items,
            intent.unwrap_or(ConfirmationIntent::Delete),
            invalid,
        )
    };

    let mut contacts = Vec::new();
    for id in &contact_ids {
        if let Some(contact) = query_as!(
            Contact,
            "SELECT id as \"id!\", contact_name, contact_user_number FROM contacts WHERE id = ?",
            id
        )
        .fetch_optional(pool)
        .await?
        {
            contacts.push(contact);
        }
    }

    match intent {
        ConfirmationIntent::AddToGroup => create_group(pool, from, contacts, invalid).await,
        ConfirmationIntent::Delete => {
            let mut tx = pool.begin().await?;
            for id in contact_ids {
                query!("DELETE FROM contacts WHERE id = ?", id)
                    .execute(&mut *tx)
                    .await?;
            }
            tx.commit().await?;

            {
                let mut pending = PENDING_ACTIONS.lock().unwrap();
                for contact in &contacts {
                    pending.retain(|_, deletion| deletion.contact_id != contact.id);
                }
            }

            let mut response = format!(
                "Deleted {} contact{}:\n",
                contacts.len(),
                if contacts.len() == 1 { "" } else { "s" }
            );

            for contact in contacts {
                let area_code = E164::from_str(&contact.contact_user_number)
                    .map(|e| e.area_code().to_string())
                    .unwrap_or_else(|_| "???".to_string());
                response.push_str(&format!("• {} ({})\n", contact.contact_name, area_code));
            }

            if !invalid.is_empty() {
                response.push_str("\nErrors:\n");
                response.push_str(&invalid.join("\n"));
            }

            Ok(response)
        }
    }
}

async fn create_group(
    pool: &Pool<Sqlite>,
    from: &str,
    contacts: Vec<Contact>,
    invalid: Vec<String>,
) -> anyhow::Result<String> {
    let mut group_num = 0;
    loop {
        let group_name = format!("group{}", group_num);
        let existing = query!(
            "SELECT id FROM groups WHERE creator_number = ? AND name = ?",
            from,
            group_name
        )
        .fetch_optional(pool)
        .await?;

        if existing.is_none() {
            break;
        }
        group_num += 1;
    }

    let group_name = format!("group{}", group_num);

    let mut tx = pool.begin().await?;

    query!(
        "INSERT INTO groups (name, creator_number) VALUES (?, ?)",
        group_name,
        from
    )
    .execute(&mut *tx)
    .await?;

    let group_id = query!(
        "SELECT id FROM groups WHERE creator_number = ? AND name = ?",
        from,
        group_name
    )
    .fetch_one(&mut *tx)
    .await?
    .id;

    for contact in &contacts {
        query!(
            "INSERT INTO group_members (group_id, member_number) VALUES (?, ?)",
            group_id,
            contact.contact_user_number
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    {
        let mut pending = PENDING_ACTIONS.lock().unwrap();
        for contact in &contacts {
            pending.retain(|_, deletion| deletion.contact_id != contact.id);
        }
    }

    let mut response = format!(
        "Created group \"{}\" with {} members:\n",
        group_name,
        contacts.len()
    );

    for contact in contacts {
        let area_code = E164::from_str(&contact.contact_user_number)
            .map(|e| e.area_code().to_string())
            .unwrap_or_else(|_| "???".to_string());
        response.push_str(&format!("• {} ({})\n", contact.contact_name, area_code));
    }

    if !invalid.is_empty() {
        response.push_str("\nErrors:\n");
        response.push_str(&invalid.join("\n"));
    }

    Ok(response)
}

fn cleanup_pending_deletions() {
    PENDING_ACTIONS
        .lock()
        .unwrap()
        .retain(|_, deletion| deletion.timestamp.elapsed() <= DELETION_TIMEOUT);
}

async fn onboard_new_user(
    command: Option<Result<Command, serde_json::Error>>,
    words: impl Iterator<Item = &str>,
    from: &str,
    pool: &Pool<Sqlite>,
) -> anyhow::Result<String> {
    let Some(Ok(Command::name)) = command else {
        return Ok(format!(
            "Greetings! This is Decision Bot (https://github.com/samcarey/decisionbot).\n\
            To participate:\n{}",
            Command::name.hint()
        ));
    };
    Ok(match process_name(words) {
        Ok(name) => {
            query!("insert into users (number, name) values (?, ?)", from, name)
                .execute(pool)
                .await?;
            format!("Hello, {name}! {}", Command::h.hint())
        }
        Err(hint) => hint.to_string(),
    })
}

fn process_name<'a>(words: impl Iterator<Item = &'a str>) -> Result<String> {
    let name = words.collect::<Vec<_>>().join(" ");
    if name.is_empty() {
        bail!("{}", Command::name.usage());
    }
    const MAX_NAME_LEN: usize = 20;
    if name.len() > MAX_NAME_LEN {
        bail!(
            "That name is {} characters long.\n\
            Please shorten it to {MAX_NAME_LEN} characters or less.",
            name.len()
        );
    }
    Ok(name)
}

async fn send(twilio_config: &Configuration, to: String, message: String) -> Result<()> {
    let message_params = CreateMessageParams {
        account_sid: env::var("TWILIO_ACCOUNT_SID")?,
        to,
        from: Some(env::var("SERVER_NUMBER")?),
        body: Some(message),
        ..Default::default()
    };
    let message = create_message(twilio_config, message_params)
        .await
        .context("While sending message")?;
    trace!("Message sent with SID {}", message.sid.unwrap().unwrap());
    Ok(())
}

enum ConfirmationIntent {
    Delete,
    AddToGroup,
}

struct PendingAction {
    contact_id: i64,
    timestamp: Instant,
    intent: ConfirmationIntent,
}

static PENDING_ACTIONS: Lazy<Mutex<HashMap<String, PendingAction>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

const DELETION_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes
