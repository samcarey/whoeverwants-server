use anyhow::{bail, Context, Result};
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
    #[allow(dead_code)]
    name: String,
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
    let SmsMessage {
        Body: body,
        From: from,
    } = message;
    debug!("Received from {from}: {body}");

    let mut words = body.trim().split_ascii_whitespace();
    let command_word = words.next();
    let command = command_word.map(|word| Command::try_from(word));

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
            if let Some(command) = command_text.map(|word| Command::try_from(word)) {
                if let Ok(command) = command {
                    format!(
                        "{} to {}.{}",
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
    };
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

#[cfg(test)]
mod test {
    use super::*;
    use futures::executor::block_on;

    fn fixture(pool: Pool<Sqlite>) -> impl Fn(&str) {
        move |message: &str| {
            println!(">'{message}'");
            let response = block_on(process_message(
                &pool,
                SmsMessage {
                    From: "TEST_NUMBER".to_string(),
                    Body: message.to_string(),
                },
            ))
            .unwrap();
            println!("{response}\n\n");
        }
    }

    #[sqlx::test]
    async fn all(pool: Pool<Sqlite>) -> Result<()> {
        let fixture = fixture(pool);

        fixture("hi");
        fixture("name Sam C.");
        fixture("h");
        fixture("info name");
        fixture("info stop");
        fixture("info  ");
        fixture("info x");
        fixture("info info");
        fixture("info name x");
        fixture("yo");
        fixture("stop");
        fixture("yo");

        Ok(())
    }
}
