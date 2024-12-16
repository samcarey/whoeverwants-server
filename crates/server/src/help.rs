use crate::{
    cleanup_expired_pending_actions, command::Command, get_pending_action_prompt, util::E164,
    CommandTrait,
};
use enum_iterator::all;
use sqlx::{Pool, Sqlite};

pub struct HelpCommand;

impl CommandTrait for HelpCommand {
    async fn handle(&self, pool: &Pool<Sqlite>, from: &E164) -> anyhow::Result<String> {
        cleanup_expired_pending_actions(pool).await?;

        let mut response = format!(
            "General commands:\n{}\n",
            all::<Command>()
                .filter(|c| match c {
                    Command::confirm => false,
                    _ => true,
                })
                .map(|c| format!("- {c}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        response.push_str(&format!("\n{}", Command::info.hint()));

        if let Some(pending_prompt) = get_pending_action_prompt(pool, from).await? {
            response.push_str(&pending_prompt);
        }

        Ok(response)
    }
}
