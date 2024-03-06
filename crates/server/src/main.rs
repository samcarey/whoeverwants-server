use anyhow::{Context, Result};
use axum::{
    response::{Html, IntoResponse},
    routing::post,
    Extension, Form, Router,
};
use dotenv::dotenv;
use enum_iterator::all;
use log::*;
use openapi::apis::{
    api20100401_message_api::{create_message, CreateMessageParams},
    configuration::Configuration,
};
use sqlx::{query, query_as, Pool, Sqlite};
use std::env;

use crate::command::Command;

mod command;

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
#[derive(serde::Deserialize)]
struct SmsMessage {
    Body: String,
    From: String,
}

struct User {
    number: String,
    name: String,
}

// Handler for incoming SMS messages
async fn handle_incoming_sms(
    Extension(pool): Extension<Pool<Sqlite>>,
    Form(message): Form<SmsMessage>,
) -> impl IntoResponse {
    let response = match process(message, &pool).await {
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

async fn process(message: SmsMessage, pool: &Pool<Sqlite>) -> anyhow::Result<String> {
    let SmsMessage {
        Body: body,
        From: from,
    } = message;
    debug!("Received from {from}: {body}");
    const HELP_HINT: &str = "Reply H to show available commands.";
    const MAX_NAME_LEN: usize = 20;
    let mut words = body.trim().split_ascii_whitespace();
    let command_word = words.next();
    let command = command_word.map(|word| Command::try_from(word));
    Ok(
        if let Some(User { number, name, .. }) =
            query_as!(User, "select * from users where number = ?", from)
                .fetch_optional(pool)
                .await?
        {
            if let Some(command) = command {
                if let Ok(command) = command {
                    match command {
                        Command::name => {
                            let name = words.collect::<Vec<_>>().join(" ");
                            if !name.is_empty() {
                                if name.len() <= MAX_NAME_LEN {
                                    query!(
                                        "update users set name = ? where number = ?",
                                        name,
                                        from
                                    )
                                    .execute(pool)
                                    .await?;
                                    format!("Your name has been updated to {name}")
                                } else {
                                    format!("That name is '{}' characters long. Please shorten it to {MAX_NAME_LEN} characters or less", name.len())
                                }
                            } else {
                                "Make sure you follow the name command with a space and then your name: 'name NAME'".to_string()
                            }
                        }
                        Command::stop => {
                            query!("delete from users where number = ?", number)
                                .execute(pool)
                                .await?;
                            // They won't actually see this when using Twilio
                            "You've been unsubscribed. Goodbye!".to_string()
                        }
                        // I would use HELP for the help command, but Twilio intercepts and does not relay that
                        Command::h => {
                            let command_text = words.next();
                            if let Some(command) = command_text.map(|word| Command::try_from(word))
                            {
                                if let Ok(command) = command {
                                    format!("{command}: {}", command.help())
                                } else {
                                    format!("Command '{}' not recognized", command_text.unwrap())
                                }
                            } else {
                                format!(
                                    "Available commands: {}. Reply 'H COMMAND' to see help for COMMAND",
                                    all::<Command>()
                                        .map(|c| c.to_string())
                                        .collect::<Vec<_>>()
                                        .join(",")
                                )
                            }
                        }
                    }
                } else {
                    format!(
                        "Hey {name}! We didn't recognize that command word: '{}'. {HELP_HINT}",
                        command_word.unwrap()
                    )
                }
            } else {
                HELP_HINT.to_string()
            }
        } else {
            match command {
                Some(Ok(Command::name)) => {
                    if let Some(name) = words.next() {
                        if name.len() <= MAX_NAME_LEN {
                            query!("insert into users (number, name) values (?, ?)", from, name)
                                .execute(pool)
                                .await?;
                            format!("Hi, {name}! {HELP_HINT}")
                        } else {
                            format!("That name is {} characters long. Please shorten it to {MAX_NAME_LEN} characters or less", name.len())
                        }
                    } else {
                        "Make sure you follow the 'name' command with a space and then your name: 'name NAME'".to_string()
                    }
                }
                _ => {
                    format!(
                        "Welcome to Sam Carey's experimental social server! \
                        To participate, reply 'name NAME', \
                        where NAME is your preferred name (max {MAX_NAME_LEN} characters)."
                    )
                }
            }
        },
    )
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
