use anyhow::Result;
use shared::util::E164;
use sqlx::{query, Pool, Sqlite};
use std::str::FromStr;

pub async fn get_pending_action_prompt(pool: &Pool<Sqlite>, from: &str) -> Result<Option<String>> {
    let pending = query!(
        "SELECT action_type FROM pending_actions WHERE submitter_number = ?",
        from
    )
    .fetch_optional(pool)
    .await?;

    match pending {
        Some(row) => {
            let prompt = match row.action_type.as_str() {
                "deletion" => {
                    let contacts = query!(
                        "SELECT c.contact_name, c.contact_user_number 
                         FROM pending_deletions pd
                         JOIN contacts c ON c.id = pd.contact_id 
                         WHERE pd.pending_action_submitter = ?
                         ORDER BY c.contact_name",
                        from
                    )
                    .fetch_all(pool)
                    .await?;

                    if contacts.is_empty() {
                        return Ok(None);
                    }

                    let list = contacts
                        .iter()
                        .enumerate()
                        .map(|(i, c)| {
                            let area_code = E164::from_str(&c.contact_user_number)
                                .map(|e| e.area_code().to_string())
                                .unwrap_or_else(|_| "???".to_string());
                            format!("{}. {} ({})", i + 1, c.contact_name, area_code)
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    format!(
                        "\n\nYou have pending contact deletions:\n{}\n\
                        To delete contacts, reply \"confirm NUM1, NUM2, ...\"",
                        list
                    )
                }
                "deferred_contacts" => {
                    let contacts = query!(
                        "SELECT DISTINCT contact_name FROM deferred_contacts 
                         WHERE submitter_number = ? 
                         ORDER BY contact_name",
                        from
                    )
                    .fetch_all(pool)
                    .await?;

                    if contacts.is_empty() {
                        return Ok(None);
                    }

                    let mut response =
                        "\n\nYou have contacts with multiple numbers pending:\n".to_string();

                    for (i, contact) in contacts.iter().enumerate() {
                        response.push_str(&format!("\n{}. {}", i + 1, contact.contact_name));

                        let numbers = query!(
                            "SELECT phone_number, phone_description 
                             FROM deferred_contacts 
                             WHERE submitter_number = ? AND contact_name = ?
                             ORDER BY id",
                            from,
                            contact.contact_name
                        )
                        .fetch_all(pool)
                        .await?;

                        for (j, number) in numbers.iter().enumerate() {
                            let letter = (b'a' + j as u8) as char;
                            let desc = number
                                .phone_description
                                .as_deref()
                                .unwrap_or("no description");
                            response.push_str(&format!(
                                "\n   {}. {} ({})",
                                letter, number.phone_number, desc
                            ));
                        }
                    }

                    response.push_str("\n\nReply with \"confirm NA, MB, ...\" where N and M are contact numbers and A and B are letter choices");
                    response
                }
                "group" => {
                    let contacts = query!(
                        "SELECT c.contact_name, c.contact_user_number 
                         FROM pending_group_members pgm
                         JOIN contacts c ON c.id = pgm.contact_id 
                         WHERE pgm.pending_action_submitter = ?
                         ORDER BY c.contact_name",
                        from
                    )
                    .fetch_all(pool)
                    .await?;

                    if contacts.is_empty() {
                        return Ok(None);
                    }

                    let list = contacts
                        .iter()
                        .enumerate()
                        .map(|(i, c)| {
                            let area_code = E164::from_str(&c.contact_user_number)
                                .map(|e| e.area_code().to_string())
                                .unwrap_or_else(|_| "???".to_string());
                            format!("{}. {} ({})", i + 1, c.contact_name, area_code)
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    format!(
                        "\n\nYou have a pending group creation:\n{}\n\
                        To create a group with these contacts, reply \"confirm NUM1, NUM2, ...\"",
                        list
                    )
                }
                _ => return Ok(None),
            };
            Ok(Some(prompt))
        }
        None => Ok(None),
    }
}
