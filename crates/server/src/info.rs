use crate::{command::Command, util::E164, CommandTrait};
use non_empty_string::NonEmptyString;
use sqlx::{Pool, Sqlite};
use std::str::FromStr;

pub struct InfoCommand {
    query: NonEmptyString,
}

impl FromStr for InfoCommand {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            query: NonEmptyString::from_str(s.trim()).map_err(|e| anyhow::format_err!("{e}"))?,
        })
    }
}

impl CommandTrait for InfoCommand {
    async fn handle(&self, _: &Pool<Sqlite>, _: &E164) -> anyhow::Result<String> {
        let Self { query } = self;
        Ok(if let Ok(command) = Command::try_from(query.as_str()) {
            format!(
                "{}, to {}.{}",
                command.usage(),
                command.description(),
                command.example()
            )
        } else {
            format!("Command \"{}\" not recognized", query)
        })
    }
}
