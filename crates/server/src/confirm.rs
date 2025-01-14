use std::str::FromStr;

use crate::{
    contacts::add_contact,
    create_group, get_pending_action_prompt,
    util::{parse_selections, ResponseBuilder, Selection, E164},
    CommandTrait, Contact, GroupRecord, ParameterDoc,
};
use sqlx::{query, query_as, Pool, Sqlite};

pub struct ConfirmCommand {
    selections: Vec<Selection>,
}

impl FromStr for ConfirmCommand {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            selections: parse_selections(s)?,
        })
    }
}

impl CommandTrait for ConfirmCommand {
    fn word() -> &'static str {
        "confirm"
    }
    fn description() -> &'static str {
        "confirm pending actions"
    }
    fn parameter_doc() -> Option<crate::ParameterDoc> {
        Some(ParameterDoc {
            example: "1,2".to_string(),
            description:
                "a comma-separated list of choices to confirm (based on a preceding list provided)"
                    .to_string(),
        })
    }
    async fn handle(&self, pool: &Pool<Sqlite>, from: &E164) -> anyhow::Result<String> {
        let Self { selections } = self;
        if selections.is_empty() {
            if let Some(pending_prompt) = get_pending_action_prompt(pool, from).await? {
                return Ok(pending_prompt);
            }
        }
        let pending_action = query!(
            "SELECT action_type FROM pending_actions WHERE submitter_number = ?",
            **from
        )
        .fetch_optional(pool)
        .await?;

        let Some(action) = pending_action else {
            return Ok("No pending actions to confirm.".to_string());
        };

        let result = match action.action_type.as_str() {
            "deferred_contacts" => handle_deferred_contacts_confirm(pool, from, selections).await,
            "deletion" => handle_deletion_confirm(pool, from, selections).await,
            "group" => handle_group_confirm(pool, from, selections).await,
            _ => Ok("Invalid action type".to_string()),
        };

        match result {
            Ok(msg) => Ok(msg),
            Err(e) => {
                let mut response = ResponseBuilder::new();
                response.add_errors(&[e.to_string()]);
                Ok(response.build())
            }
        }
    }
}

pub async fn handle_deferred_contacts_confirm(
    pool: &Pool<Sqlite>,
    from: &E164,
    selections: &[Selection],
) -> anyhow::Result<String> {
    let mut successful: Vec<String> = Vec::new();
    let mut failed: Vec<String> = Vec::new();

    // Get all deferred contacts
    let deferred_contacts = query!(
        "SELECT DISTINCT contact_name FROM deferred_contacts WHERE submitter_number = ?",
        **from
    )
    .fetch_all(pool)
    .await?;

    for selection in selections {
        // Get the contact name
        let Some(contact_name) = deferred_contacts
            .get(selection.index)
            .map(|c| &c.contact_name)
        else {
            // This case matches the test expectation exactly
            return Ok(format!("Contact number {} not found", selection.index + 1));
        };

        // Get all numbers for this contact to validate letter selection
        let numbers = query!(
            "SELECT phone_number, phone_description FROM deferred_contacts 
             WHERE submitter_number = ? AND contact_name = ?
             ORDER BY id",
            **from,
            contact_name
        )
        .fetch_all(pool)
        .await?;

        let Some(letter) = selection.sub_selection else {
            failed.push(format!(
                "No letter selection for contact {}",
                selection.index + 1
            ));
            continue;
        };

        let letter_idx = (letter as u8 - b'a') as usize;
        if letter_idx >= numbers.len() {
            return Ok(format!("Invalid letter selection: {}", letter));
        }

        // Get the selected number
        let number = &numbers[letter_idx];

        // Insert the contact
        if let Err(e) = add_contact(pool, from, contact_name, &number.phone_number).await {
            failed.push(format!(
                "Failed to add {} ({}): {}",
                contact_name, number.phone_number, e
            ));
        } else {
            successful.push(format!("{} ({})", contact_name, number.phone_number));
        }
    }

    // If we get here and have any failures, return the first failure
    if !failed.is_empty() {
        return Ok(failed[0].clone());
    }

    // Only clean up and format success message if everything succeeded
    if !successful.is_empty() {
        let mut tx = pool.begin().await?;

        // Remove all deferred contacts for this submitter
        query!(
            "DELETE FROM deferred_contacts WHERE submitter_number = ?",
            **from
        )
        .execute(&mut *tx)
        .await?;

        // Clean up pending action
        query!(
            "DELETE FROM pending_actions WHERE submitter_number = ?",
            **from
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let contact_text = if successful.len() == 1 {
            "contact"
        } else {
            "contacts"
        };
        return Ok(format!(
            "Successfully added {} {}:\n{}\n",
            successful.len(),
            contact_text,
            successful.join("\n")
        ));
    }

    Ok("No changes made.".to_string())
}

pub async fn handle_deletion_confirm(
    pool: &Pool<Sqlite>,
    from: &str,
    selections: &[Selection],
) -> anyhow::Result<String> {
    let mut deleted_groups: Vec<String> = Vec::new();
    let mut deleted_contacts: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    // Get all pending items
    let groups = query_as!(
        GroupRecord,
        r#"WITH member_counts AS (
            SELECT 
                group_id,
                COUNT(*) as count
            FROM group_members
            GROUP BY group_id
        )
        SELECT 
            g.id as "id!", 
            g.name,
            COALESCE(mc.count, 0) as "member_count!"
        FROM groups g
        JOIN pending_deletions pd ON pd.group_id = g.id
        LEFT JOIN member_counts mc ON mc.group_id = g.id
        WHERE pd.pending_action_submitter = ?
        ORDER BY g.name"#,
        from
    )
    .fetch_all(pool)
    .await?;

    let contacts = query_as!(
        Contact,
        "SELECT c.id as \"id!\", c.contact_name, c.contact_user_number 
         FROM contacts c
         JOIN pending_deletions pd ON pd.contact_id = c.id
         WHERE pd.pending_action_submitter = ?
         ORDER BY c.contact_name",
        from
    )
    .fetch_all(pool)
    .await?;

    // Process selections
    for selection in selections {
        let idx = selection.index;
        if idx < groups.len() {
            let group = &groups[idx];
            deleted_groups.push(format!("{} ({} members)", group.name, group.member_count));
        } else if idx < groups.len() + contacts.len() {
            let contact = &contacts[idx - groups.len()];
            let area_code = E164::from_str(&contact.contact_user_number)
                .map(|e| e.area_code().to_string())
                .unwrap_or_else(|_| "???".to_string());
            deleted_contacts.push(format!("{} ({})", contact.contact_name, area_code));
        } else {
            errors.push(format!("Invalid selection: {}", idx + 1));
        }
    }

    if deleted_groups.is_empty() && deleted_contacts.is_empty() && errors.is_empty() {
        return Ok("No valid selections provided.".to_string());
    }

    let mut tx = pool.begin().await?;

    // Delete selected groups and their members
    for (i, _) in deleted_groups.iter().enumerate() {
        let group = &groups[i];
        query!("DELETE FROM group_members WHERE group_id = ?", group.id)
            .execute(&mut *tx)
            .await?;
        query!("DELETE FROM groups WHERE id = ?", group.id)
            .execute(&mut *tx)
            .await?;
    }

    // Delete selected contacts
    for (i, _) in deleted_contacts.iter().enumerate() {
        let contact = &contacts[i];
        query!("DELETE FROM contacts WHERE id = ?", contact.id)
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

    let mut response = String::new();

    // Format group deletion confirmation
    if !deleted_groups.is_empty() {
        let group_text = if deleted_groups.len() == 1 {
            "group"
        } else {
            "groups"
        };
        response.push_str(&format!(
            "Deleted {} {}:\n{}\n",
            deleted_groups.len(),
            group_text,
            deleted_groups.join("\n")
        ));
    }

    // Format contact deletion confirmation
    if !deleted_contacts.is_empty() {
        if !response.is_empty() {
            response.push_str("\n");
        }
        let contact_text = if deleted_contacts.len() == 1 {
            "contact"
        } else {
            "contacts"
        };
        response.push_str(&format!(
            "Deleted {} {}:\n{}\n",
            deleted_contacts.len(),
            contact_text,
            deleted_contacts.join("\n")
        ));
    }

    // Add any errors
    if !errors.is_empty() {
        if !response.is_empty() {
            response.push_str("\n");
        }
        response.push_str("Errors:\n");
        response.push_str(&errors.join("\n"));
    }

    Ok(response)
}

pub async fn handle_group_confirm(
    pool: &Pool<Sqlite>,
    from: &str,
    selections: &[Selection],
) -> anyhow::Result<String> {
    let mut selected_contacts: Vec<Contact> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for selection in selections {
        let offset = selection.index as i64;
        match query!(
            "SELECT c.id as \"id!\", c.contact_name, c.contact_user_number 
             FROM contacts c
             JOIN pending_group_members pgm ON pgm.contact_id = c.id
             WHERE pgm.pending_action_submitter = ?
             ORDER BY c.contact_name
             LIMIT 1 OFFSET ?",
            from,
            offset
        )
        .fetch_optional(pool)
        .await?
        {
            Some(row) => selected_contacts.push(Contact {
                id: row.id,
                contact_name: row.contact_name,
                contact_user_number: row.contact_user_number,
            }),
            None => errors.push(format!("Invalid selection: {}", selection.index + 1)),
        }
    }

    create_group(pool, from, selected_contacts, errors).await
}
