use crate::{Contact, User};
use anyhow::Result;
use sqlx::{query_as, Pool, Sqlite};

pub async fn get_user(pool: &Pool<Sqlite>, number: &str) -> Result<Option<User>> {
    query_as!(User, "SELECT * FROM users WHERE number = ?", number)
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
}

pub async fn get_contacts_by_name(
    pool: &Pool<Sqlite>,
    submitter: &str,
    name_fragment: &str,
) -> Result<Vec<Contact>> {
    let like = format!("%{}%", name_fragment.to_lowercase());
    query_as!(
        Contact,
        "SELECT id as \"id!\", contact_name, contact_user_number 
         FROM contacts 
         WHERE submitter_number = ? 
         AND LOWER(contact_name) LIKE ?
         ORDER BY contact_name",
        submitter,
        like
    )
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}
