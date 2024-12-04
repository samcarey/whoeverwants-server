use std::fmt::Display;

use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

// variants must be all lowercase for serde_json to deserialize them
#[allow(non_camel_case_types)]
#[derive(Deserialize, Serialize, Sequence, Debug)]
pub(crate) enum Command {
    h,
    name,
    info,
    stop,
    friends,
    delete,
    confirm,
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

struct ParameterDoc {
    example: String,
    description: String,
}
impl Command {
    pub fn description(&self) -> String {
        match self {
            Self::h => "show a list of available commands",
            Self::info => "see information about a command",
            Self::name => "set your preferred name",
            Self::stop => "stop receiving messages and remove yourself from the database",
            Self::friends => "see a list of your stored contacts",
            Self::delete => "delete a contact by name",
            Self::confirm => "confirm deletion of a contact",
        }
        .to_string()
    }

    fn parameter_doc(&self) -> Option<ParameterDoc> {
        match self {
            Self::h => None,
            Self::info => Some(ParameterDoc {
                example: Command::name.to_string(),
                description: "a command".to_string(),
            }),
            Self::name => Some(ParameterDoc {
                example: "John S.".to_string(),
                description: "your name".to_string(),
            }),
            Self::delete => Some(ParameterDoc {
                example: "John".to_string(),
                description: "contact name to delete".to_string(),
            }),
            Self::confirm => Some(ParameterDoc {
                example: "2".to_string(),
                description: "number from the deletion list".to_string(),
            }),
            Self::stop => None,
            Self::friends => None,
        }
    }
    pub fn usage(&self) -> String {
        if let Some(ParameterDoc { description, .. }) = self.parameter_doc() {
            format!("Reply \"{self} X\", where X is {description},")
        } else {
            format!("Reply \"{self}\"")
        }
    }
    pub fn example(&self) -> String {
        self.parameter_doc()
            .map(|ParameterDoc { example, .. }| format!("\nExample: \"{self} {example}\""))
            .unwrap_or_default()
    }
    pub fn hint(&self) -> String {
        format!(
            "{}, to {}.{}",
            self.usage(),
            self.description(),
            self.example()
        )
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
