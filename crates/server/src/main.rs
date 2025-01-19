use anyhow::{Context, Ok, Result};
use axum::{routing::post, Extension, Router};
use dotenv::dotenv;
use log::*;
use openapi::apis::{
    api20100401_message_api::{create_message, CreateMessageParams},
    configuration::Configuration,
};
use sms::handle_incoming_sms;
use sqlx::{query, Pool, Sqlite};
use std::env;
use std::str::FromStr;
use util::E164;

mod contacts;
mod sms;
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
        .route("/sms", post(handle_incoming_sms))
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

struct User {
    #[allow(unused)]
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
