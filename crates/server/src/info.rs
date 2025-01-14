use crate::{command::Command, util::E164, CommandTrait, ParameterDoc};
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
    fn word() -> &'static str {
        "info"
    }
    fn description() -> &'static str {
        "see information about a command"
    }
    fn parameter_doc() -> Option<crate::ParameterDoc> {
        Some(ParameterDoc {
            example: Command::name.to_string(),
            description: "a command".to_string(),
        })
    }
    async fn handle(&self, _: &Pool<Sqlite>, _: &E164) -> anyhow::Result<String> {
        let Self { query } = self;
        Ok(if let Ok(command) = Command::try_from(query.as_str()) {
            format!(
                "{}, to {}.{}",
                Self::usage(),
                Self::description(),
                Self::example()
            )
        } else {
            format!("Command \"{}\" not recognized", query)
        })
    }
}
