use crate::{command::Command, util::E164, CommandTrait};
use anyhow::{bail, Result};
use non_empty_string::NonEmptyString;
use sqlx::{query, Pool, Sqlite};
use std::str::FromStr;

pub struct NameCommand {
    name: NonEmptyString,
}

impl FromStr for NameCommand {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            name: NonEmptyString::from_str(s.trim()).map_err(|e| anyhow::format_err!("{e}"))?,
        })
    }
}

impl CommandTrait for NameCommand {
    async fn handle(&self, pool: &Pool<Sqlite>, from: &E164) -> anyhow::Result<String> {
        let Self { name } = self;
        validate_name(&name.to_string())?;
        let name = name.to_string();
        query!("update users set name = ? where number = ?", name, **from)
            .execute(pool)
            .await?;
        Ok(format!("Your name has been updated to \"{name}\""))
    }
}

pub fn validate_name<'a>(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("{}", Command::name.usage());
    }
    const MAX_NAME_LEN: usize = 20;
    if name.len() > MAX_NAME_LEN {
        bail!(
            "That name is {} characters long.\n\
            Please shorten it to {MAX_NAME_LEN} characters or less.",
            name.len()
        );
    }
    Ok(())
}
