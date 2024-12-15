use crate::{
    db::get_contacts_by_name,
    set_pending_action,
    util::{format_contact_list, ResponseBuilder},
};
use sqlx::{query, Pool, Sqlite};

pub async fn handle_group(pool: &Pool<Sqlite>, from: &str, names: &str) -> anyhow::Result<String> {
    let name_fragments: Vec<_> = names.split(',').map(str::trim).collect();

    if name_fragments.is_empty() {
        return Ok("Please provide at least one name to search for.".to_string());
    }

    let mut contacts = Vec::new();
    for fragment in &name_fragments {
        let mut matches = get_contacts_by_name(pool, from, fragment).await?;
        contacts.append(&mut matches);
    }

    contacts.sort_by(|a, b| a.id.cmp(&b.id));
    contacts.dedup_by(|a, b| a.id == b.id);
    contacts.sort_by(|a, b| a.contact_name.cmp(&b.contact_name));

    if contacts.is_empty() {
        return Ok(format!(
            "No contacts found matching: {}",
            name_fragments.join(", ")
        ));
    }

    let mut tx = pool.begin().await?;
    set_pending_action(pool, from, "group", &mut tx).await?;

    for contact in &contacts {
        query!(
            "INSERT INTO pending_group_members (pending_action_submitter, contact_id) 
             VALUES (?, ?)",
            from,
            contact.id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let mut response = ResponseBuilder::new();
    response.add_section("Found these contacts", format_contact_list(&contacts, 0));
    response.add_section(
        "",
        "To create a group with these contacts, reply \"confirm NUM1, NUM2, ...\"".to_string(),
    );

    Ok(response.build())
}
