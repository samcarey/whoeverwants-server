use anyhow::Result;
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
    Ok(())
}
