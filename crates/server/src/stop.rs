use std::str::FromStr;

use sqlx::query;

use crate::CommandTrait;

pub struct StopCommand;

impl FromStr for StopCommand {
    type Err = anyhow::Error;
    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Ok(Self)
    }
}

impl CommandTrait for StopCommand {
    async fn handle(
        &self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        from: &crate::util::E164,
    ) -> anyhow::Result<String> {
        query!("delete from users where number = ?", **from)
            .execute(pool)
            .await?;
        // They won't actually see this when using Twilio
        Ok("You've been unsubscribed. Goodbye!".to_string())
    }
}
