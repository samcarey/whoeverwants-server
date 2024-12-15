use sqlx::{query, query_as, Pool, Sqlite};
use std::str::FromStr;

use crate::{contacts::add_contact, create_group, util::E164, Contact, GroupRecord};

pub async fn handle_confirm(
    pool: &Pool<Sqlite>,
    from: &str,
    selections: &str,
) -> anyhow::Result<String> {
    let pending_action = query!(
        "SELECT action_type FROM pending_actions WHERE submitter_number = ?",
        from
    )
    .fetch_optional(pool)
    .await?;

    let Some(action) = pending_action else {
        return Ok("No pending actions to confirm.".to_string());
    };

    let action_type = action.action_type;

    match action_type.as_str() {
        "deferred_contacts" => {
            let mut successful = Vec::new();
            let mut failed = Vec::new();

            // Get all deferred contacts
            let deferred_contacts = query!(
                "SELECT DISTINCT contact_name FROM deferred_contacts WHERE submitter_number = ?",
                from
            )
            .fetch_all(pool)
            .await?;

            // Process selections like "1a, 2b, 3a"
            for selection in selections.split(',').map(str::trim) {
                // First validate basic format: must be digits followed by a single letter
                if !selection
                    .chars()
                    .rev()
                    .next()
                    .map(|c| c.is_ascii_lowercase())
                    .unwrap_or(false)
                    || !selection[..selection.len() - 1]
                        .chars()
                        .all(|c| c.is_ascii_digit())
                {
                    failed.push(format!("Invalid selection format: {}", selection));
                    continue;
                }

                // Split into numeric and letter parts
                let (num_str, letter) = selection.split_at(selection.len() - 1);
                let contact_idx: usize = match num_str.parse::<usize>() {
                    Ok(n) if n > 0 => n - 1,
                    _ => {
                        failed.push(format!("Invalid contact number: {}", num_str));
                        continue;
                    }
                };

                // Get the contact name
                let Some(contact_name) =
                    deferred_contacts.get(contact_idx).map(|c| &c.contact_name)
                else {
                    failed.push(format!("Contact number {} not found", contact_idx + 1));
                    continue;
                };

                // Get all numbers for this contact to validate letter selection
                let numbers = query!(
                    "SELECT phone_number, phone_description FROM deferred_contacts 
             WHERE submitter_number = ? AND contact_name = ?
             ORDER BY id",
                    from,
                    contact_name
                )
                .fetch_all(pool)
                .await?;

                let letter = letter.chars().next().unwrap();
                let letter_idx = match letter {
                    'a'..='z' => {
                        let idx = (letter as u8 - b'a') as usize;
                        if idx >= numbers.len() {
                            failed.push(format!("Invalid letter selection: {}", letter));
                            continue;
                        }
                        idx
                    }
                    _ => {
                        failed.push(format!("Invalid letter selection: {}", letter));
                        continue;
                    }
                };

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

            // Clean up processed contacts
            let mut tx = pool.begin().await?;
            for contact in &successful {
                if let Some(name) = contact.split(" (").next() {
                    query!(
                        "DELETE FROM deferred_contacts WHERE submitter_number = ? AND contact_name = ?",
                        from,
                        name
                    )
                    .execute(&mut *tx)
                    .await?;
                }
            }

            // Clean up pending action if all contacts are processed
            let remaining = query!(
                "SELECT COUNT(*) as count FROM deferred_contacts WHERE submitter_number = ?",
                from
            )
            .fetch_one(&mut *tx)
            .await?;

            if remaining.count == 0 {
                query!(
                    "DELETE FROM pending_actions WHERE submitter_number = ?",
                    from
                )
                .execute(&mut *tx)
                .await?;
            }

            tx.commit().await?;

            // Format response
            let mut response = String::new();
            if !successful.is_empty() {
                response.push_str(&format!(
                    "Successfully added {} contact{}:\n",
                    successful.len(),
                    if successful.len() == 1 { "" } else { "s" }
                ));
                for contact in successful {
                    response.push_str(&format!("• {}\n", contact));
                }
            }

            if !failed.is_empty() {
                if !response.is_empty() {
                    response.push_str("\n");
                }
                response.push_str("Failed to process:\n");
                for error in failed {
                    response.push_str(&format!("• {}\n", error));
                }
            }

            Ok(response)
        }
        "deletion" => {
            let mut invalid = Vec::new();
            let mut selected_groups = Vec::new();
            let mut selected_contacts = Vec::new();

            // Get total number of pending items to validate selection range
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
            for num_str in selections.split(',').map(str::trim) {
                match num_str.parse::<usize>() {
                    Ok(num) if num > 0 => {
                        let num = num - 1; // Convert to 0-based index
                        if num < groups.len() {
                            selected_groups.push(GroupRecord {
                                id: groups[num].id,
                                name: groups[num].name.clone(),
                                member_count: groups[num].member_count,
                            });
                        } else if num < groups.len() + contacts.len() {
                            selected_contacts.push(contacts[num - groups.len()].clone());
                        } else {
                            invalid.push(format!("Invalid selection: {}", num + 1));
                        }
                    }
                    _ => invalid.push(format!("Invalid selection: {}", num_str)),
                }
            }

            if selected_groups.is_empty() && selected_contacts.is_empty() && invalid.is_empty() {
                return Ok("No valid selections provided.".to_string());
            }

            let mut tx = pool.begin().await?;

            // Delete selected groups
            for group in &selected_groups {
                query!("DELETE FROM groups WHERE id = ?", group.id)
                    .execute(&mut *tx)
                    .await?;
            }

            // Delete selected contacts
            for contact in &selected_contacts {
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

            // Format response
            let mut response = String::new();

            if !selected_groups.is_empty() {
                response.push_str(&format!(
                    "Deleted {} group{}:\n",
                    selected_groups.len(),
                    if selected_groups.len() == 1 { "" } else { "s" }
                ));
                for group in selected_groups {
                    response.push_str(&format!(
                        "• {} ({} members)\n",
                        group.name, group.member_count
                    ));
                }
            }
            if !selected_contacts.is_empty() {
                if !response.is_empty() {
                    response.push_str("\n");
                }
                response.push_str(&format!(
                    "Deleted {} contact{}:\n",
                    selected_contacts.len(),
                    if selected_contacts.len() == 1 {
                        ""
                    } else {
                        "s"
                    }
                ));
                for contact in selected_contacts {
                    let area_code = E164::from_str(&contact.contact_user_number)
                        .map(|e| e.area_code().to_string())
                        .unwrap_or_else(|_| "???".to_string());
                    response.push_str(&format!("• {} ({})\n", contact.contact_name, area_code));
                }
            }

            if !invalid.is_empty() {
                if !response.is_empty() {
                    response.push_str("\n");
                }
                response.push_str("Errors:\n");
                response.push_str(&invalid.join("\n"));
            }

            Ok(response)
        }
        "group" => {
            // Existing group creation logic remains the same
            let mut invalid = Vec::new();
            let mut selected_contacts = Vec::new();

            for num_str in selections.split(',').map(str::trim) {
                match num_str.parse::<usize>() {
                    Ok(num) if num > 0 => {
                        let offset = (num - 1) as i64;
                        let query = query!(
                            "SELECT c.id as \"id!\", c.contact_name, c.contact_user_number 
                             FROM contacts c
                             JOIN pending_group_members pgm ON pgm.contact_id = c.id
                             WHERE pgm.pending_action_submitter = ?
                             ORDER BY c.contact_name
                             LIMIT 1 OFFSET ?",
                            from,
                            offset
                        );
                        if let Some(row) = query.fetch_optional(pool).await? {
                            selected_contacts.push(Contact {
                                id: row.id,
                                contact_name: row.contact_name,
                                contact_user_number: row.contact_user_number,
                            });
                        } else {
                            invalid.push(format!("Invalid selection: {}", num));
                        }
                    }
                    _ => invalid.push(format!("Invalid number: {}", num_str)),
                }
            }

            create_group(pool, from, selected_contacts, invalid).await
        }
        _ => Ok("Invalid action type".to_string()),
    }
}
