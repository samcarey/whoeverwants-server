use std::str::FromStr;

use anyhow::{bail, Result};
use ical::parser::vcard::component::VcardContact;
use sqlx::{query, Pool, Sqlite};

use crate::{util::E164, ImportResult};

pub async fn process_contact_submission(
    pool: &Pool<Sqlite>,
    from: &str,
    media_url: &Option<String>,
) -> anyhow::Result<String> {
    let vcard_data = reqwest::get(media_url.as_ref().unwrap())
        .await?
        .text()
        .await?;
    let reader = ical::VcardParser::new(vcard_data.as_bytes());
    let mut stats = ImportStats::default();

    for vcard in reader {
        match process_vcard(pool, &from, vcard).await {
            Ok(ImportResult::Added) => stats.added += 1,
            Ok(ImportResult::Updated) => stats.updated += 1,
            Ok(ImportResult::Unchanged) => stats.skipped += 1,
            Ok(ImportResult::Deferred) => stats.deferred += 1,
            Err(e) => stats.add_error(&e.to_string()),
        }
    }
    stats.format_report(pool, &from).await
}

pub async fn process_vcard(
    pool: &Pool<Sqlite>,
    from: &str,
    vcard: Result<VcardContact, ical::parser::ParserError>,
) -> Result<ImportResult> {
    let user_exists = query!("SELECT * FROM users WHERE number = ?", from)
        .fetch_optional(pool)
        .await?
        .is_some();
    if !user_exists {
        bail!("Please set your name first using the 'name' command before adding contacts");
    }

    let card = vcard?;

    let name = card
        .properties
        .iter()
        .find(|p| p.name == "FN")
        .and_then(|p| p.value.as_ref())
        .ok_or_else(|| anyhow::anyhow!("No name provided"))?;

    // Collect all TEL properties with their types/descriptions
    let mut numbers = Vec::new();
    for prop in card.properties.iter().filter(|p| p.name == "TEL") {
        if let Some(raw_number) = &prop.value {
            if let Ok(normalized) = E164::from_str(raw_number) {
                let description = prop.params.as_ref().and_then(|params| {
                    params
                        .iter()
                        .find(|(key, _)| key.eq_ignore_ascii_case("TYPE"))
                        .and_then(|(_, values)| values.first())
                        .map(|v| v.to_string())
                });
                numbers.push((normalized.to_string(), description));
            }
        }
    }

    if numbers.is_empty() {
        bail!("No valid phone numbers provided");
    }

    // Check existing contacts
    let existing_contacts = query!(
        "SELECT contact_user_number, contact_name FROM contacts WHERE submitter_number = ?",
        from
    )
    .fetch_all(pool)
    .await?;

    // If any number matches an existing contact, update that contact's name and return
    for (num, _) in &numbers {
        if let Some(existing) = existing_contacts
            .iter()
            .find(|contact| &contact.contact_user_number == num)
        {
            if existing.contact_name != *name {
                query!(
                    "UPDATE contacts SET contact_name = ? WHERE submitter_number = ? AND contact_user_number = ?",
                    name,
                    from,
                    num
                )
                .execute(pool)
                .await?;
                return Ok(ImportResult::Updated);
            }
            return Ok(ImportResult::Unchanged);
        }
    }

    if numbers.len() > 1 {
        // Store numbers in deferred_contacts table
        let mut tx = pool.begin().await?;

        // First clear any existing deferred contacts for this submitter and contact name
        query!(
            "DELETE FROM deferred_contacts WHERE submitter_number = ? AND contact_name = ?",
            from,
            name
        )
        .execute(&mut *tx)
        .await?;

        // Insert all numbers as deferred contacts
        for (number, description) in numbers {
            query!(
                "INSERT INTO deferred_contacts (submitter_number, contact_name, phone_number, phone_description) 
                 VALUES (?, ?, ?, ?)",
                from,
                name,
                number,
                description
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(ImportResult::Deferred)
    } else {
        // Single number case - proceed with insertion
        let (number, _) = numbers.into_iter().next().unwrap();
        add_contact(pool, from, name, &number).await?;
        Ok(ImportResult::Added)
    }
}

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

// Update ImportStats to include deferred count
#[derive(Default)]
struct ImportStats {
    added: usize,
    updated: usize,
    skipped: usize,
    failed: usize,
    deferred: usize,
    errors: std::collections::HashMap<String, usize>,
}

impl ImportStats {
    fn add_error(&mut self, error: &str) {
        *self.errors.entry(error.to_string()).or_insert(0) += 1;
        self.failed += 1;
    }

    async fn format_report(&self, pool: &Pool<Sqlite>, from: &str) -> Result<String> {
        let mut report = format!(
            "Processed contacts: {} added, {} updated, {} unchanged, {} deferred, {} failed",
            self.added, self.updated, self.skipped, self.deferred, self.failed
        );

        if !self.errors.is_empty() {
            report.push_str("\nErrors encountered:");
            for (error, count) in &self.errors {
                report.push_str(&format!("\n- {} × {}", count, error));
            }
        }

        if self.deferred > 0 {
            // Get all unique contact names for this submitter
            let contacts = query!(
                "SELECT DISTINCT contact_name FROM deferred_contacts WHERE submitter_number = ? ORDER BY contact_name",
                from
            )
            .fetch_all(pool)
            .await?;

            if !contacts.is_empty() {
                report.push_str(
                    "\n\nThe following contacts have multiple numbers. \
                    Reply with \"pick NA, MB, ...\" \
                    where N and M are from the list of contacts below \
                    and A and B are the letters for the desired phone numbers for each.\n",
                );

                for (i, contact) in contacts.iter().enumerate() {
                    report.push_str(&format!("\n{}. {}", i + 1, contact.contact_name));

                    // Get all numbers for this contact
                    let numbers = query!(
                        "SELECT phone_number, phone_description FROM deferred_contacts 
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
                        report.push_str(&format!(
                            "\n   {}. {} ({})",
                            letter, number.phone_number, desc
                        ));
                    }
                }
            }
        }
        Ok(report)
    }
}
