use anyhow::Result;
use sqlx::{query, Pool, Sqlite};

pub async fn add_contact(pool: &Pool<Sqlite>, from: &str, name: &str, number: &str) -> Result<()> {
    let mut tx = pool.begin().await?;

    // Create user if needed
    let contact_user = query!("SELECT * FROM users WHERE number = ?", number)
        .fetch_optional(&mut *tx)
        .await?;

    if contact_user.is_none() {
        query!(
            "INSERT INTO users (number, name) VALUES (?, ?)",
            number,
            name
        )
        .execute(&mut *tx)
        .await?;
    }

    // Insert contact
    query!(
        "INSERT INTO contacts (submitter_number, contact_name, contact_user_number) 
         VALUES (?, ?, ?)",
        from,
        name,
        number
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
