use crate::command::Command;
use anyhow::{Context, Result};
use axum::{
    response::{Html, IntoResponse},
    routing::post,
    Extension, Form, Router,
};
use confirm::ConfirmCommand;
use contacts::{process_contact_submission, ContactsCommand};
use delete::DeleteCommand;
use dotenv::dotenv;
use group::GroupCommand;
use help::HelpCommand;
use info::InfoCommand;
use log::*;
use name::{validate_name, NameCommand};
use openapi::apis::{
    api20100401_message_api::{create_message, CreateMessageParams},
    configuration::Configuration,
};
use sqlx::{query, Pool, Sqlite};
use std::env;
use std::str::FromStr;
use util::{ResponseBuilder, E164};

mod command;
mod confirm;
mod contacts;
mod db;
mod delete;
mod group;
mod help;
mod info;
mod name;
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

trait CommandTrait {
    async fn handle(&self, pool: &Pool<Sqlite>, from: &E164) -> anyhow::Result<String>;
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
        NumMedia: media_count,
        MediaContentType0: media_type_0,
        MediaUrl0: media_url_0,
    } = message;
    let from = E164::from_str(&from)?;
    debug!("Received from {from}: {body}");
    if media_count == Some("1".to_string())
        && media_type_0
            .map(|t| ["text/vcard", "text/x-vcard"].contains(&t.as_str()))
            .unwrap_or(false)
    {
        return process_contact_submission(pool, &from, &media_url_0).await;
    }

    let mut words = body.trim().split_ascii_whitespace();
    let command_word = words.next();
    let command = command_word.map(Command::try_from);

    let Some(User { number, .. }) = db::get_user(pool, &from).await? else {
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
        Command::h => HelpCommand.handle(pool, &from).await?,
        Command::name => {
            let name = words.collect::<Vec<_>>().join(" ");
            match NameCommand::from_str(&name) {
                Ok(command) => command.handle(pool, &from).await?,
                Err(error) => {
                    let mut response = ResponseBuilder::new();
                    response.add_errors(&[error.to_string()]);
                    response.add_section(&Command::name.hint());
                    response.build()
                }
            }
        }
        Command::stop => {
            query!("delete from users where number = ?", number)
                .execute(pool)
                .await?;
            // They won't actually see this when using Twilio
            "You've been unsubscribed. Goodbye!".to_string()
        }
        Command::info => {
            let command_text = words.next().unwrap_or_default();
            match InfoCommand::from_str(command_text) {
                Ok(command) => command.handle(pool, &from).await?,
                Err(error) => {
                    let mut response = ResponseBuilder::new();
                    response.add_errors(&[error.to_string()]);
                    response.add_section(&Command::info.hint());
                    response.build()
                }
            }
        }
        Command::contacts => ContactsCommand.handle(pool, &from).await?,
        Command::delete => {
            let name = words.collect::<Vec<_>>().join(" ");
            match DeleteCommand::from_str(&name) {
                Ok(command) => command.handle(pool, &from).await?,
                Err(error) => {
                    let mut response = ResponseBuilder::new();
                    response.add_errors(&[error.to_string()]);
                    response.add_section(&Command::delete.hint());
                    response.build()
                }
            }
        }
        Command::confirm => {
            let nums = words.collect::<Vec<_>>().join(" ");
            match ConfirmCommand::from_str(&nums) {
                Ok(command) => command.handle(pool, &from).await?,
                Err(error) => {
                    let mut response = ResponseBuilder::new();
                    response.add_errors(&[error.to_string()]);
                    response.add_section(&Command::confirm.hint());
                    response.build()
                }
            }
        }
        Command::group => {
            let names = words.collect::<Vec<_>>().join(" ");
            match GroupCommand::from_str(&names) {
                Ok(command) => command.handle(pool, &from).await?,
                Err(error) => {
                    let mut response = ResponseBuilder::new();
                    response.add_errors(&[error.to_string()]);
                    response.add_section(&Command::group.hint());
                    response.build()
                }
            }
        }
    };
    Ok(response)
}

#[derive(Clone, sqlx::FromRow)]
struct GroupRecord {
    id: i64,
    name: String,
    member_count: i64,
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
    from: &E164,
    pool: &Pool<Sqlite>,
) -> anyhow::Result<String> {
    let Some(Ok(Command::name)) = command else {
        return Ok(format!(
            "Greetings! This is Decision Bot (https://github.com/samcarey/decisionbot).\n\
            To participate:\n{}",
            Command::name.hint()
        ));
    };
    let name = words.collect::<Vec<_>>().join(" ");
    Ok(match validate_name(&name) {
        Ok(()) => {
            query!(
                "insert into users (number, name) values (?, ?)",
                **from,
                name
            )
            .execute(pool)
            .await?;
            format!("Hello, {name}! {}", Command::h.hint())
        }
        Err(hint) => hint.to_string(),
    })
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

async fn get_pending_action_prompt(pool: &Pool<Sqlite>, from: &E164) -> Result<Option<String>> {
    let pending = query!(
        "SELECT action_type FROM pending_actions WHERE submitter_number = ?",
        **from
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
                        **from
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
                        **from
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
                            **from,
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
                        **from
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
