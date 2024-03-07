use std::fmt::Display;

use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

// variants must be all lowercase for serde_json to deserialize them
#[allow(non_camel_case_types)]
#[derive(Deserialize, Serialize, Sequence, Debug)]
pub(crate) enum Command {
    h,
    name,
    stop,
}

impl TryFrom<&str> for Command {
    type Error = serde_json::Error;
    fn try_from(value: &str) -> std::prelude::v1::Result<Self, Self::Error> {
        serde_json::from_str(&format!("\"{}\"", value.to_lowercase()))
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).split("::").last().unwrap())
    }
}

impl Command {
    pub fn help(&self) -> String {
        format!(
            "{}.{}",
            match self {
                Self::h => "Show a list of available commands",
                Self::name => "Set your preferred name",
                Self::stop => "Stop receiving messages and remove yourself from the database",
            },
            self.example()
        )
    }
    pub fn example(&self) -> String {
        format!(
            "\nExample: '{}'",
            match self {
                Self::h => format!("h {}", Self::name),
                Self::name => "name John S.".to_string(),
                Self::stop => "stop".to_string(),
            }
        )
    }
    pub fn usage(&self) -> String {
        let mut args = vec![self.to_string()];
        match self {
            Self::h => args.push("<command>".to_string()),
            Self::name => args.push("<name>".to_string()),
            Self::stop => {}
        }
        args.join(" ").replace("<", "&lt").replace(">", "&gt")
    }
}

#[test]
fn command() {
    let command_text = "name";
    assert_eq!(
        Command::try_from(command_text).unwrap().to_string(),
        command_text
    );
}
