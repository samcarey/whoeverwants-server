use anyhow::{Context, Result};
use axum::{
    response::{Html, IntoResponse},
    routing::post,
    Extension, Form, Router,
};
use dotenv::dotenv;
use openapi::apis::{
    api20100401_message_api::{create_message, CreateMessageParams},
    configuration::Configuration,
};
use sqlx::{query, query_as, Pool, Sqlite};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;
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
    println!("listening on {}", listener.local_addr()?);
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
            println!("Error: {error}");
            "Internal Server Error!".to_string()
        }
    };
    println!("Sending response: {response}");
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
    println!("Received from {from}: {body}");
    const HELP_HINT: &str = "Reply HELP to show available commands.";
    const MAX_NAME_LEN: usize = 20;
    let mut words = body.trim().split_ascii_whitespace();
    let command = words.next().map(|word| word.to_lowercase());
    Ok(
        if let Some(User { number, name, .. }) =
            query_as!(User, "select * from users where number = ?", from)
                .fetch_optional(pool)
                .await?
        {
            if let Some(command) = command.as_deref() {
                match command {
                    "stop" => {
                        query!("delete from users where number = ?", number)
                            .execute(pool)
                            .await?;
                        format!("You've been unsubscribed. Goodbye!")
                    }
                    command => {
                        format!(
                        "Hey {name}! We didn't recognize that command word: '{command}'. {HELP_HINT}"
                    )
                    }
                }
            } else {
                HELP_HINT.to_string()
            }
        } else {
            match (command.as_deref(), words.next()) {
                (Some("start"), Some(name)) if name.len() <= MAX_NAME_LEN => {
                    query!("insert into users (number, name) values (?, ?)", from, name)
                        .execute(pool)
                        .await?;
                    format!("Thank you for participating, {name}!. {HELP_HINT}")
                }
                _ => {
                    format!(
                    "Welcome to Sam Carey's experimental social server. To participate, reply 'START NAME', where NAME is your preferred name (max {MAX_NAME_LEN} characters)."
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
    println!("Message sent with SID {}", message.sid.unwrap().unwrap());
    Ok(())
}

// // Make our own error that wraps `anyhow::Error`.
// struct AppError(anyhow::Error);

// // Tell axum how to convert `AppError` into a response.
// impl IntoResponse for AppError {
//     fn into_response(self) -> Response {
//         (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             format!("Something went wrong: {}", self.0),
//         )
//             .into_response()
//     }
// }

// // This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// // `Result<_, AppError>`. That way you don't need to do that manually.
// impl<E> From<E> for AppError
// where
//     E: Into<anyhow::Error>,
// {
//     fn from(err: E) -> Self {
//         Self(err.into())
//     }
// }
