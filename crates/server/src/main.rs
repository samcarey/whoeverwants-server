use anyhow::Result;
use axum::{
    response::{Html, IntoResponse},
    routing::post,
    Form, Router,
};
use dotenv::dotenv;
use openapi::apis::{
    api20100401_message_api::{create_message, CreateMessageParams},
    configuration::Configuration,
};
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

    let message_params = CreateMessageParams {
        account_sid: env::var("TWILIO_ACCOUNT_SID")?,
        to: env::var("CLIENT_NUMBER")?,
        from: Some(env::var("SERVER_NUMBER")?),
        body: Some("Ahoy again x3, Rustacean! 🦀".into()),
        ..Default::default()
    };
    let message = create_message(&twilio_config, message_params).await;
    match message {
        Ok(result) => {
            println!("Message sent with SID {}", result.sid.unwrap().unwrap())
        }
        Err(error) => eprintln!("Error sending message: {}", error),
    };

    let app = Router::new().route("/", post(handle_incoming_sms));

    // run it
    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        env::var("CALLBACK_IP")?,
        env::var("CALLBACK_PORT")?
    ))
    .await
    .unwrap();
    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

// Define a struct to deserialize the incoming form data
#[derive(serde::Deserialize, serde::Serialize)]
struct SmsMessage {
    body: String,
}

// Handler for incoming SMS messages
async fn handle_incoming_sms(Form(SmsMessage { body }): Form<SmsMessage>) -> impl IntoResponse {
    Html(format!(
        r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Response>
        <Message>Thank you for your submission: {body}</Message>
        </Response>
        "#
    ))
}
