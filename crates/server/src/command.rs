use std::str::FromStr;

use crate::{
    cleanup_expired_pending_actions, command_word::CommandWord, contacts::add_contact,
    create_group, help::get_pending_action_prompt, set_pending_action, util::E164, Contact,
    GroupRecord,
};
use anyhow::{bail, Context};
use enum_iterator::all;
use sqlx::{query, query_as, Pool, Sqlite};

pub enum Command {
    Help,
    Name { name: String },
    Info { command_word: CommandWord },
    Stop,
    Contacts,
    Delete { name: String },
    Confirm { selections: Vec<String> },
    Group { selections: Vec<String> },
}

pub trait FromCommandWord {
    fn to_command<'a>(self, words: impl Iterator<Item = &'a str>) -> anyhow::Result<Command>;
}

impl FromCommandWord for CommandWord {
    fn to_command<'a>(self, mut words: impl Iterator<Item = &'a str>) -> anyhow::Result<Command> {
        let command = match self {
            CommandWord::h => Command::Help,
            CommandWord::name => {
                let name = words.collect::<Vec<_>>().join(" ");
                if name.is_empty() {
                    bail!("{}", CommandWord::name.usage());
                }
                const MAX_NAME_LEN: usize = 20;
                if name.len() > MAX_NAME_LEN {
                    bail!(
                        "That name is {} characters long.\n\
                        Please shorten it to {MAX_NAME_LEN} characters or less.",
                        name.len()
                    );
                }
                Command::Name { name }
            }
            CommandWord::info => {
                let command_word = words.next().context(CommandWord::info.hint())?;
                let command_word = CommandWord::try_from(command_word)
                    .context(format!("Command \"{command_word}\" not recognized"))?;
                Command::Info { command_word }
            }
            CommandWord::stop => Command::Stop,
            CommandWord::confirm => Command::Confirm {
                selections: words
                    .collect::<Vec<_>>()
                    .join("")
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
            },
            CommandWord::contacts => Command::Contacts,
            CommandWord::delete => Command::Delete {
                name: format!("%{}%", words.collect::<Vec<_>>().join(" ").to_lowercase()),
            },
            CommandWord::group => Command::Group {
                selections: words
                    .collect::<Vec<_>>()
                    .join("")
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
            },
        };
        Ok(command)
    }
}

impl Command {
    pub async fn handle(&self, pool: &Pool<Sqlite>, from: &str) -> anyhow::Result<String> {
        let message = match self {
            Command::Help => {
                cleanup_expired_pending_actions(pool).await?;
                let mut response = format!(
                    "General commands:\n{}\n",
                    all::<CommandWord>()
                        .filter(|c| match c {
                            CommandWord::confirm => false,
                            _ => true,
                        })
                        .map(|c| format!("- {c}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
                response.push_str(&format!("\n{}", CommandWord::info.hint()));

                if let Some(pending_prompt) = get_pending_action_prompt(pool, from).await? {
                    response.push_str(&pending_prompt);
                }
                response
            }
            Command::Name { name } => {
                query!("update users set name = ? where number = ?", name, from)
                    .execute(pool)
                    .await?;
                format!("Your name has been updated to \"{name}\"")
            }
            Command::Info { command_word } => {
                format!(
                    "{}, to {}.{}",
                    command_word.usage(),
                    command_word.description(),
                    command_word.example()
                )
            }
            Command::Confirm { selections } => handle_confirm(pool, from, selections).await?,
            Command::Group { selections } => handle_group(pool, from, selections).await?,
            Command::Delete { name } => handle_delete(pool, from, name).await?,
            Command::Stop => {
                query!("delete from users where number = ?", from)
                    .execute(pool)
                    .await?;
                // They won't actually see this when using Twilio
                "You've been unsubscribed. Goodbye!".to_string()
            }
            Command::Contacts => handle_contacts(pool, from).await?,
        };
        Ok(message)
    }
}

async fn handle_confirm(
    pool: &Pool<Sqlite>,
    from: &str,
    selections: &Vec<String>,
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
            for selection in selections {
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
            for num_str in selections {
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

            for num_str in selections {
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

async fn handle_group(
    pool: &Pool<Sqlite>,
    from: &str,
    name_fragments: &Vec<String>,
) -> anyhow::Result<String> {
    if name_fragments.is_empty() {
        return Ok("Please provide at least one name to search for.".to_string());
    }

    let mut contacts = Vec::new();
    for fragment in name_fragments {
        let like = format!("%{}%", fragment.to_lowercase());
        let mut matches = query_as!(
            Contact,
            "SELECT id as \"id!\", contact_name, contact_user_number 
             FROM contacts 
             WHERE submitter_number = ? 
             AND LOWER(contact_name) LIKE ?
             ORDER BY contact_name",
            from,
            like
        )
        .fetch_all(pool)
        .await?;
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

    // Set pending action type to group
    set_pending_action(pool, from, "group", &mut tx).await?;

    // Store contacts for group creation
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

    Ok(format!(
        "Found these contacts:\n{}\n\nTo create a group with these contacts, reply \"confirm NUM1, NUM2, ...\"",
        list
    ))
}

async fn handle_delete(pool: &Pool<Sqlite>, from: &str, query: &str) -> anyhow::Result<String> {
    // Find matching groups
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
            groups.id as "id!", 
            groups.name,
            COALESCE(mc.count, 0) as "member_count!"
        FROM groups 
        LEFT JOIN member_counts mc ON mc.group_id = groups.id
        WHERE creator_number = ? 
        AND LOWER(name) LIKE ?
        ORDER BY name"#,
        from,
        query
    )
    .fetch_all(pool)
    .await?;
    // Find matching contacts (rest of the code unchanged)
    let contacts = query_as!(
        Contact,
        "SELECT id as \"id!\", contact_name, contact_user_number 
         FROM contacts 
         WHERE submitter_number = ? 
         AND LOWER(contact_name) LIKE ?
         ORDER BY contact_name",
        from,
        query
    )
    .fetch_all(pool)
    .await?;

    if groups.is_empty() && contacts.is_empty() {
        return Ok(format!(
            "No groups or contacts found matching \"{}\"",
            query
        ));
    }

    let mut tx = pool.begin().await?;

    // Set pending action type to deletion
    set_pending_action(pool, from, "deletion", &mut tx).await?;

    // Store groups for deletion
    for group in &groups {
        query!(
            "INSERT INTO pending_deletions (pending_action_submitter, group_id, contact_id) 
             VALUES (?, ?, NULL)",
            from,
            group.id
        )
        .execute(&mut *tx)
        .await?;
    }

    // Store contacts for deletion
    for contact in &contacts {
        query!(
            "INSERT INTO pending_deletions (pending_action_submitter, group_id, contact_id) 
             VALUES (?, NULL, ?)",
            from,
            contact.id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let mut response = String::new();

    // List groups if any were found
    if !groups.is_empty() {
        response.push_str("Found these groups:\n");
        for (i, group) in groups.iter().enumerate() {
            response.push_str(&format!(
                "{}. {} ({} members)\n",
                i + 1,
                group.name,
                group.member_count
            ));
        }
    }

    // List contacts if any were found, continuing the numbering
    if !contacts.is_empty() {
        if !groups.is_empty() {
            response.push_str("\n");
        }
        response.push_str("Found these contacts:\n");
        let offset = groups.len();
        for (i, c) in contacts.iter().enumerate() {
            let area_code = E164::from_str(&c.contact_user_number)
                .map(|e| e.area_code().to_string())
                .unwrap_or_else(|_| "???".to_string());

            response.push_str(&format!(
                "{}. {} ({})\n",
                i + offset + 1,
                c.contact_name,
                area_code
            ));
        }
    }

    response.push_str(
        "\nTo delete items, reply \"confirm NUM1, NUM2, ...\", \
        where NUM1, NUM2, etc. are numbers from the lists above.",
    );

    Ok(response)
}

async fn handle_contacts(pool: &Pool<Sqlite>, from: &str) -> anyhow::Result<String> {
    // First get the groups
    let groups = query!(
        "SELECT g.name, COUNT(gm.member_number) as member_count 
         FROM groups g 
         LEFT JOIN group_members gm ON g.id = gm.group_id
         WHERE g.creator_number = ?
         GROUP BY g.id, g.name
         ORDER BY g.name",
        from
    )
    .fetch_all(pool)
    .await?;

    // Then get the contacts
    let contacts = query_as!(
        Contact,
        "SELECT id as \"id!\", contact_name, contact_user_number 
         FROM contacts 
         WHERE submitter_number = ? 
         ORDER BY contact_name",
        from
    )
    .fetch_all(pool)
    .await?;

    Ok(if groups.is_empty() && contacts.is_empty() {
        "You don't have any groups or contacts.".to_string()
    } else {
        let mut response = String::new();

        // Add groups section if there are any
        if !groups.is_empty() {
            response.push_str("Your groups:\n");
            for (i, group) in groups.iter().enumerate() {
                response.push_str(&format!(
                    "{}. {} ({} members)\n",
                    i + 1,
                    group.name,
                    group.member_count
                ));
            }
        }

        // Add contacts section if there are any
        if !contacts.is_empty() {
            if !groups.is_empty() {
                response.push_str("\n"); // Add spacing between sections
            }
            response.push_str("Your contacts:\n");
            let offset = groups.len(); // Start contact numbering after groups
            response.push_str(
                &contacts
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        format!(
                            "{}. {} ({})",
                            i + offset + 1,
                            c.contact_name,
                            &E164::from_str(&c.contact_user_number)
                                .expect("Should have been formatted upon db insertion")
                                .area_code()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }
        response
    })
}
