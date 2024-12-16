use crate::{set_pending_action, util::E164, CommandTrait, Contact, GroupRecord};
use non_empty_string::NonEmptyString;
use sqlx::{query, query_as, Pool, Sqlite};
use std::str::FromStr;

pub struct DeleteCommand {
    name: NonEmptyString,
}

impl FromStr for DeleteCommand {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            name: NonEmptyString::from_str(s).map_err(|e| anyhow::format_err!("{e}"))?,
        })
    }
}

impl CommandTrait for DeleteCommand {
    async fn handle(&self, pool: &Pool<Sqlite>, from: &E164) -> anyhow::Result<String> {
        let Self { name } = self;
        let like = format!("%{}%", name.to_string().to_lowercase());

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
            **from,
            like
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
            **from,
            like
        )
        .fetch_all(pool)
        .await?;

        if groups.is_empty() && contacts.is_empty() {
            return Ok(format!("No groups or contacts found matching \"{}\"", name));
        }

        let mut tx = pool.begin().await?;

        // Set pending action type to deletion
        set_pending_action(pool, from, "deletion", &mut tx).await?;

        // Store groups for deletion
        for group in &groups {
            query!(
                "INSERT INTO pending_deletions (pending_action_submitter, group_id, contact_id) 
             VALUES (?, ?, NULL)",
                **from,
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
                **from,
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
}
