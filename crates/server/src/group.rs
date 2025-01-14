use crate::{
    db::get_contacts_by_name,
    set_pending_action,
    util::{format_contact_list, ResponseBuilder, E164},
    CommandTrait, ParameterDoc,
};
use sqlx::{query, Pool, Sqlite};
use std::str::FromStr;

pub struct GroupCommand {
    name_fragments: Vec<String>,
}

impl FromStr for GroupCommand {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            name_fragments: s
                .split(',')
                .map(str::trim)
                .map(ToString::to_string)
                .collect(),
        })
    }
}

impl CommandTrait for GroupCommand {
    fn word() -> &'static str {
        "group"
    }
    fn description() -> &'static str {
        "create a new group from your contacts"
    }
    fn parameter_doc() -> Option<crate::ParameterDoc> {
        Some(ParameterDoc {
            example: "John, Alice".to_string(),
            description: "comma-separated list of contact name fragments".to_string(),
        })
    }
    async fn handle(&self, pool: &Pool<Sqlite>, from: &E164) -> anyhow::Result<String> {
        let Self { name_fragments } = self;

        if name_fragments.is_empty() {
            return Ok("Please provide at least one name to search for.".to_string());
        }

        let mut contacts = Vec::new();
        for fragment in name_fragments {
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
                **from,
                contact.id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        let mut response = ResponseBuilder::new();
        response.add_titled("Found these contacts", format_contact_list(&contacts, 0));
        response.add_section(
            "To create a group with these contacts, reply \"confirm NUM1, NUM2, ...\"",
        );

        Ok(response.build())
    }
}
