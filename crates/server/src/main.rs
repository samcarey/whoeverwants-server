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

use crate::{
    command::Command,
    friends::{accept, friend, reject},
};

mod command;
mod friends;

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
        Some(&twilio_config),
        env::var("CLIENT_NUMBER")?,
        "Server is starting up".to_string(),
    )
    .await?;
    let pool = sqlx::SqlitePool::connect(&env::var("DATABASE_URL")?).await?;
    let app = Router::new()
        .route("/", post(handle_incoming_sms))
        .layer(Extension(pool))
        .layer(Extension(twilio_config));
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
    #[allow(dead_code)]
    number: String,
    name: String,
}

struct RowId {
    rowid: i64,
}

struct Friendship {
    rowid: i64,
    number_a: String,
    number_b: String,
}

// Handler for incoming SMS messages
async fn handle_incoming_sms(
    Extension(pool): Extension<Pool<Sqlite>>,
    Extension(twilio_config): Extension<Configuration>,
    Form(message): Form<SmsMessage>,
) -> impl IntoResponse {
    let response = match process_message(&pool, Some(&twilio_config), message).await {
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

async fn process_message(
    pool: &Pool<Sqlite>,
    twilio_config: Option<&Configuration>,
    message: SmsMessage,
) -> anyhow::Result<String> {
    let SmsMessage {
        Body: body,
        From: from,
    } = message;
    debug!("Received from {from}: {body}");

    let mut words = body.trim().split_ascii_whitespace();
    let command_word = words.next();
    let command = command_word.map(|word| Command::try_from(word));

    let Some(User { name, .. }) = query_as!(User, "select * from users where number = ?", from)
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
            "We didn't recognize that command word: '{}'.\n{}",
            command_word.unwrap(),
            Command::h.hint()
        ));
    };

    query!("begin transaction").execute(pool).await?;

    let response = match (command, words.next()) {
        // I would use HELP for the help command, but Twilio intercepts and does not relay that
        (Command::h, _) => {
            let available_commands = format!(
                "Available commands:\n{}\n",
                all::<Command>()
                    .map(|c| format!("- {c}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            format!("{available_commands}\n{}", Command::info.hint())
        }
        (Command::name, _) => match process_name(words) {
            Ok(name) => {
                query!("update users set name = ? where number = ?", name, from)
                    .execute(pool)
                    .await?;
                format!("Your name has been updated to '{name}'")
            }
            Err(hint) => hint.to_string(),
        },
        (Command::stop, _) => {
            query!("delete from users where number = ?", from)
                .execute(pool)
                .await?;
            // They won't actually see this when using Twilio
            "You've been unsubscribed. Goodbye!".to_string()
        }
        (Command::info, Some(command_word)) => {
            if let Ok(command) = Command::try_from(command_word) {
                format!(
                    "{} to {}.{}",
                    command.usage(),
                    command.description(),
                    command.example()
                )
            } else {
                format!("Command '{command_word}' not recognized")
            }
        }
        (Command::friend, Some(friend_number)) => {
            // Look for all combos of number, and sanitize
            friend(pool, twilio_config, &name, &from, friend_number).await?
        }
        (Command::unfriend, Some(friend_index)) => {
            // todo only compare index once
            if let Some(Friendship {
                rowid,
                number_a,
                number_b,
            }) = query_as!(
                Friendship,
                "select rowid, number_a, number_b from friendships where \
                (number_a, rowid) = (?, ?) or \
                (number_b, rowid) = (?, ?)",
                from,
                friend_index,
                from,
                friend_index
            )
            .fetch_optional(pool)
            .await?
            {
                let friend_number = if number_a == from { number_b } else { number_a };
                let friend_name = query!("select name from users where number = ?", friend_number)
                    .fetch_one(pool)
                    .await?
                    .name;
                query!("delete from friendships where rowid = ?", rowid)
                    .execute(pool)
                    .await?;
                format!("Removed {friend_name} ({friend_number}) as a friend")
            } else {
                format!("Failed to find that friendship")
            }
        }
        (Command::unfriend, None) => {
            let results = query_as!(
                Friendship,
                "select rowid, number_a, number_b from friendships where \
                number_a = ? or number_b = ?",
                from,
                from,
            )
            .fetch_all(pool)
            .await?;
            let mut friends = Vec::with_capacity(results.len());
            for Friendship {
                rowid,
                number_a,
                number_b,
            } in results
            {
                let friend_number = if number_a == from { number_b } else { number_a };
                let friend_name = query!("select name from users where number = ?", friend_number)
                    .fetch_one(pool)
                    .await?
                    .name;
                friends.push(format!("{rowid}: {friend_name}"));
            }
            format!("Friendships:\n{}", friends.join("\n"))
        }
        (Command::accept | Command::reject | Command::block, Some(request_index)) => {
            if let Ok(request_index) = request_index.parse() {
                match command {
                    Command::accept => {
                        accept(pool, twilio_config, &name, &from, request_index).await?
                    }
                    Command::reject => reject(pool, &from, request_index).await?,
                    Command::block => {
                        unimplemented!()
                    }
                    _ => unreachable!(),
                }
            } else {
                command.hint()
            }
        }
        (Command::unblock, Some(request_index)) => {
            unimplemented!()
        }
        (Command::unblock, None) => {
            unimplemented!()
        }
        (Command::accept | Command::reject | Command::block, None) => {
            unimplemented!()
        }
        (command, None) => command.hint(),
    };
    query!("commit").execute(pool).await?;
    Ok(response.replace("'", "\""))
}

async fn onboard_new_user(
    command: Option<Result<Command, serde_json::Error>>,
    words: impl Iterator<Item = &str>,
    from: &str,
    pool: &Pool<Sqlite>,
) -> anyhow::Result<String> {
    let Some(Ok(Command::name)) = command else {
        return Ok(format!(
            "Welcome to Sam Carey's experimental social server!\nTo participate:\n{}",
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

async fn send(twilio_config: Option<&Configuration>, to: String, message: String) -> Result<()> {
    if let Some(twilio_config) = twilio_config {
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
    } else {
        println!("To ({to}): {message}\n\n");
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::executor::block_on;

    fn create_fixture(pool: &Pool<Sqlite>, number: &str) -> impl Fn(&str) {
        let number = number.to_string();
        let pool = pool.clone();
        move |message: &str| {
            println!("{number} >>> '{message}'");
            let response = block_on(process_message(
                &pool,
                None,
                SmsMessage {
                    From: number.to_string(),
                    Body: message.to_string(),
                },
            ))
            .unwrap();
            println!("{response}\n\n");
        }
    }

    fn one_sided(pool: Pool<Sqlite>, input: &[&str]) {
        let fixture = create_fixture(&pool, "TEST_NUMBER");
        for i in input {
            fixture(i);
        }
    }

    #[sqlx::test]
    async fn basic(pool: Pool<Sqlite>) {
        one_sided(
            pool,
            &[
                "hi",
                "name Sam C.",
                "h",
                "info name",
                "info stop",
                "info  ",
                "info x",
                "info info",
                "info name x",
                "yo",
                "stop",
                "yo",
            ],
        );
    }

    #[sqlx::test]
    async fn manage_friends(pool: Pool<Sqlite>) {
        let a = create_fixture(&pool, "A");
        let b = create_fixture(&pool, "B");
        a("name Sam C.");
        a("friend B");
        b("accept");
        b("accept 1");
    }
}
