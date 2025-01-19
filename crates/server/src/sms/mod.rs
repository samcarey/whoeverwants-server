use crate::User;
use anyhow::{bail, Result};
use axum::{
    response::{Html, IntoResponse},
    Extension, Form,
};
use command::FromCommandWord;
use command_word::CommandWord;
use contacts::process_contact_submission;
use log::{debug, error};
use sqlx::{query, query_as, Pool, Sqlite};

mod command;
mod command_word;
mod contacts;
mod help;
#[cfg(test)]
mod test;

pub async fn handle_incoming_sms(
    Extension(pool): Extension<Pool<Sqlite>>,
    Form(message): Form<SmsMessage>,
) -> impl IntoResponse {
    let response = match process_message(&pool, message).await {
        anyhow::Result::Ok(response) => response,
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
    log::trace!("Received {message:?}");
    let SmsMessage {
        Body: body,
        From: from,
        NumMedia: media_count,
        MediaContentType0: media_type_0,
        MediaUrl0: media_url_0,
    } = message;
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
    let command = command_word.map(CommandWord::try_from);

    if query_as!(User, "select * from users where number = ?", from)
        .fetch_optional(pool)
        .await?
        .is_none()
    {
        return onboard_new_user(command, words, &from, pool).await;
    }

    let Some(command) = command else {
        return Ok(CommandWord::h.hint());
    };
    let anyhow::Result::Ok(command_word) = command else {
        return Ok(format!(
            "We didn't recognize that command word: \"{}\".\n{}",
            command_word.unwrap(),
            CommandWord::h.hint()
        ));
    };

    let command = match command_word.to_command(words) {
        anyhow::Result::Ok(command) => command,
        Err(error) => {
            return Ok(error.to_string());
        }
    };
    command.handle(pool, &from).await
}

// field names must be exact (including case) to match API
#[allow(non_snake_case)]
#[derive(serde::Deserialize, Default, Debug)]
pub struct SmsMessage {
    Body: String,
    From: String,
    NumMedia: Option<String>,
    MediaContentType0: Option<String>,
    MediaUrl0: Option<String>,
}

async fn onboard_new_user(
    command: Option<Result<CommandWord, serde_json::Error>>,
    words: impl Iterator<Item = &str>,
    from: &str,
    pool: &Pool<Sqlite>,
) -> anyhow::Result<String> {
    let Some(anyhow::Result::Ok(CommandWord::name)) = command else {
        return Ok(format!(
            "Greetings! This is Decision Bot (https://github.com/samcarey/decisionbot).\n\
            To participate:\n{}",
            CommandWord::name.hint()
        ));
    };
    Ok(match process_name(words) {
        anyhow::Result::Ok(name) => {
            query!("insert into users (number, name) values (?, ?)", from, name)
                .execute(pool)
                .await?;
            format!("Hello, {name}! {}", CommandWord::h.hint())
        }
        Err(hint) => hint.to_string(),
    })
}

fn process_name<'a>(words: impl Iterator<Item = &'a str>) -> Result<String> {
    let name = words.collect::<Vec<_>>().join(" ");
    if name.is_empty() {
        bail!("{}", CommandWord::name.usage());
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
