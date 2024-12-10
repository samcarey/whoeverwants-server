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
use openapi::apis::{
    api20100401_message_api::{create_message, CreateMessageParams},
    configuration::Configuration,
};
use sqlx::{query, query_as, Pool, Sqlite};
use std::env;
use std::str::FromStr;
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
        Command::h => handle_help(pool, &from).await?,
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

async fn handle_help(pool: &Pool<Sqlite>, from: &str) -> Result<String> {
    cleanup_expired_pending_actions(pool).await?;

    let mut response = format!(
        "Available commands:\n{}\n",
        all::<Command>()
            .map(|c| format!("- {c}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    response.push_str(&format!("\n{}", Command::info.hint()));

    if let Some(pending_prompt) = get_pending_action_prompt(pool, from).await? {
        response.push_str(&pending_prompt);
    }

    Ok(response)
}

async fn handle_group(pool: &Pool<Sqlite>, from: &str, names: &str) -> anyhow::Result<String> {
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

    let mut tx = pool.begin().await?;

    // Set pending action type to group
    set_pending_action(pool, from, "group", &mut tx).await?;

    // Store contacts for group creation
    for contact in &contacts {
        query!(
            "INSERT INTO pending_group_members (pending_action_submitter, contact_id) 
             VALUES (?, ?)",
            from,
            contact.id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

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

    Ok(format!(
        "Found these contacts:\n{}\n\nTo create a group with these contacts, reply \"confirm NUM1, NUM2, ...\"",
        list
    ))
}

async fn handle_delete(pool: &Pool<Sqlite>, from: &str, name: &str) -> anyhow::Result<String> {
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

    let mut tx = pool.begin().await?;

    // Set pending action type to deletion
    set_pending_action(pool, from, "deletion", &mut tx).await?;

    // Store contacts for deletion
    for contact in &contacts {
        query!(
            "INSERT INTO pending_deletions (pending_action_submitter, contact_id) 
             VALUES (?, ?)",
            from,
            contact.id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

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
    let pending_action = query!(
        "SELECT action_type FROM pending_actions WHERE submitter_number = ?",
        from
    )
    .fetch_optional(pool)
    .await?;

    let Some(action) = pending_action else {
        return Ok("No pending actions to confirm.".to_string());
    };

    let action_type = action.action_type;

    match action_type.as_str() {
        "deferred_contacts" => {
            let mut successful = Vec::new();
            let mut failed = Vec::new();

            // Get all deferred contacts
            let deferred_contacts = query!(
                "SELECT DISTINCT contact_name FROM deferred_contacts WHERE submitter_number = ?",
                from
            )
            .fetch_all(pool)
            .await?;

            // Process selections like "1a, 2b, 3a"
            for selection in selections.split(',').map(str::trim) {
                // First validate basic format: must be digits followed by a single letter
                if !selection
                    .chars()
                    .rev()
                    .next()
                    .map(|c| c.is_ascii_lowercase())
                    .unwrap_or(false)
                    || !selection[..selection.len() - 1]
                        .chars()
                        .all(|c| c.is_ascii_digit())
                {
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

                // Get the contact name
                let Some(contact_name) =
                    deferred_contacts.get(contact_idx).map(|c| &c.contact_name)
                else {
                    failed.push(format!("Contact number {} not found", contact_idx + 1));
                    continue;
                };

                // Get all numbers for this contact to validate letter selection
                let numbers = query!(
                    "SELECT phone_number, phone_description FROM deferred_contacts 
             WHERE submitter_number = ? AND contact_name = ?
             ORDER BY id",
                    from,
                    contact_name
                )
                .fetch_all(pool)
                .await?;

                let letter = letter.chars().next().unwrap();
                let letter_idx = match letter {
                    'a'..='z' => {
                        let idx = (letter as u8 - b'a') as usize;
                        if idx >= numbers.len() {
                            failed.push(format!("Invalid letter selection: {}", letter));
                            continue;
                        }
                        idx
                    }
                    _ => {
                        failed.push(format!("Invalid letter selection: {}", letter));
                        continue;
                    }
                };

                // Get the selected number
                let number = &numbers[letter_idx];

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
            let mut tx = pool.begin().await?;
            for contact in &successful {
                if let Some(name) = contact.split(" (").next() {
                    query!(
                        "DELETE FROM deferred_contacts WHERE submitter_number = ? AND contact_name = ?",
                        from,
                        name
                    )
                    .execute(&mut *tx)
                    .await?;
                }
            }

            // Clean up pending action if all contacts are processed
            let remaining = query!(
                "SELECT COUNT(*) as count FROM deferred_contacts WHERE submitter_number = ?",
                from
            )
            .fetch_one(&mut *tx)
            .await?;

            if remaining.count == 0 {
                query!(
                    "DELETE FROM pending_actions WHERE submitter_number = ?",
                    from
                )
                .execute(&mut *tx)
                .await?;
            }

            tx.commit().await?;

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
        "deletion" => {
            // Existing deletion logic remains the same
            let mut invalid = Vec::new();
            let mut selected_contacts = Vec::new();

            for num_str in selections.split(',').map(str::trim) {
                match num_str.parse::<usize>() {
                    Ok(num) if num > 0 => {
                        let offset = (num - 1) as i64;
                        let query = query!(
                            "SELECT c.id as \"id!\", c.contact_name, c.contact_user_number 
                             FROM contacts c
                             JOIN pending_deletions pd ON pd.contact_id = c.id
                             WHERE pd.pending_action_submitter = ?
                             ORDER BY c.contact_name
                             LIMIT 1 OFFSET ?",
                            from,
                            offset
                        );
                        if let Some(row) = query.fetch_optional(pool).await? {
                            selected_contacts.push(Contact {
                                id: row.id,
                                contact_name: row.contact_name,
                                contact_user_number: row.contact_user_number,
                            });
                        } else {
                            invalid.push(format!("Invalid selection: {}", num));
                        }
                    }
                    _ => invalid.push(format!("Invalid number: {}", num_str)),
                }
            }

            if selected_contacts.is_empty() && invalid.is_empty() {
                return Ok("No valid selections provided.".to_string());
            }

            let mut tx = pool.begin().await?;

            for contact in &selected_contacts {
                query!("DELETE FROM contacts WHERE id = ?", contact.id)
                    .execute(&mut *tx)
                    .await?;
            }

            // Clean up pending actions
            query!(
                "DELETE FROM pending_actions WHERE submitter_number = ?",
                from
            )
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;

            let mut response = format!(
                "Deleted {} contact{}:\n",
                selected_contacts.len(),
                if selected_contacts.len() == 1 {
                    ""
                } else {
                    "s"
                }
            );

            for contact in selected_contacts {
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
        "group" => {
            // Existing group creation logic remains the same
            let mut invalid = Vec::new();
            let mut selected_contacts = Vec::new();

            for num_str in selections.split(',').map(str::trim) {
                match num_str.parse::<usize>() {
                    Ok(num) if num > 0 => {
                        let offset = (num - 1) as i64;
                        let query = query!(
                            "SELECT c.id as \"id!\", c.contact_name, c.contact_user_number 
                             FROM contacts c
                             JOIN pending_group_members pgm ON pgm.contact_id = c.id
                             WHERE pgm.pending_action_submitter = ?
                             ORDER BY c.contact_name
                             LIMIT 1 OFFSET ?",
                            from,
                            offset
                        );
                        if let Some(row) = query.fetch_optional(pool).await? {
                            selected_contacts.push(Contact {
                                id: row.id,
                                contact_name: row.contact_name,
                                contact_user_number: row.contact_user_number,
                            });
                        } else {
                            invalid.push(format!("Invalid selection: {}", num));
                        }
                    }
                    _ => invalid.push(format!("Invalid number: {}", num_str)),
                }
            }

            create_group(pool, from, selected_contacts, invalid).await
        }
        _ => Ok("Invalid action type".to_string()),
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

    // Clean up pending actions
    query!(
        "DELETE FROM pending_actions WHERE submitter_number = ?",
        from
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

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

async fn cleanup_expired_pending_actions(pool: &Pool<Sqlite>) -> Result<()> {
    query!("DELETE FROM pending_actions WHERE created_at < unixepoch() - 300")
        .execute(pool)
        .await?;
    Ok(())
}

async fn get_pending_action_prompt(pool: &Pool<Sqlite>, from: &str) -> Result<Option<String>> {
    let pending = query!(
        "SELECT action_type FROM pending_actions WHERE submitter_number = ?",
        from
    )
    .fetch_optional(pool)
    .await?;

    match pending {
        Some(row) => {
            let prompt = match row.action_type.as_str() {
                "deletion" => {
                    let contacts = query!(
                        "SELECT c.contact_name, c.contact_user_number 
                         FROM pending_deletions pd
                         JOIN contacts c ON c.id = pd.contact_id 
                         WHERE pd.pending_action_submitter = ?
                         ORDER BY c.contact_name",
                        from
                    )
                    .fetch_all(pool)
                    .await?;

                    if contacts.is_empty() {
                        return Ok(None);
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

                    format!(
                        "\n\nYou have pending contact deletions:\n{}\n\
                        To delete contacts, reply \"confirm NUM1, NUM2, ...\"",
                        list
                    )
                }
                "deferred_contacts" => {
                    let contacts = query!(
                        "SELECT DISTINCT contact_name FROM deferred_contacts 
                         WHERE submitter_number = ? 
                         ORDER BY contact_name",
                        from
                    )
                    .fetch_all(pool)
                    .await?;

                    if contacts.is_empty() {
                        return Ok(None);
                    }

                    let mut response =
                        "\n\nYou have contacts with multiple numbers pending:\n".to_string();

                    for (i, contact) in contacts.iter().enumerate() {
                        response.push_str(&format!("\n{}. {}", i + 1, contact.contact_name));

                        let numbers = query!(
                            "SELECT phone_number, phone_description 
                             FROM deferred_contacts 
                             WHERE submitter_number = ? AND contact_name = ?
                             ORDER BY id",
                            from,
                            contact.contact_name
                        )
                        .fetch_all(pool)
                        .await?;

                        for (j, number) in numbers.iter().enumerate() {
                            let letter = (b'a' + j as u8) as char;
                            let desc = number
                                .phone_description
                                .as_deref()
                                .unwrap_or("no description");
                            response.push_str(&format!(
                                "\n   {}. {} ({})",
                                letter, number.phone_number, desc
                            ));
                        }
                    }

                    response.push_str("\n\nReply with \"confirm NA, MB, ...\" where N and M are contact numbers and A and B are letter choices");
                    response
                }
                "group" => {
                    let contacts = query!(
                        "SELECT c.contact_name, c.contact_user_number 
                         FROM pending_group_members pgm
                         JOIN contacts c ON c.id = pgm.contact_id 
                         WHERE pgm.pending_action_submitter = ?
                         ORDER BY c.contact_name",
                        from
                    )
                    .fetch_all(pool)
                    .await?;

                    if contacts.is_empty() {
                        return Ok(None);
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

                    format!(
                        "\n\nYou have a pending group creation:\n{}\n\
                        To create a group with these contacts, reply \"confirm NUM1, NUM2, ...\"",
                        list
                    )
                }
                _ => return Ok(None),
            };
            Ok(Some(prompt))
        }
        None => Ok(None),
    }
}

async fn set_pending_action(
    _pool: &Pool<Sqlite>, // Changed to _pool since it's unused
    from: &str,
    action_type: &str,
    tx: &mut sqlx::Transaction<'_, Sqlite>,
) -> Result<()> {
    // Clear any existing pending action
    query!(
        "DELETE FROM pending_actions WHERE submitter_number = ?",
        from
    )
    .execute(&mut **tx)
    .await?;

    // Create new pending action
    query!(
        "INSERT INTO pending_actions (submitter_number, action_type) VALUES (?, ?)",
        from,
        action_type
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}
