use crate::Contact;
use anyhow::{bail, Result};
use std::fmt::Display;
use std::str::FromStr;

/// E164 phone number format validator and parser
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct E164(String);

impl E164 {
    /// Returns the area code (NPA) portion of the phone number
    pub fn area_code(&self) -> &str {
        &self.0[2..5]
    }

    /// Returns the full E164 formatted string
    #[allow(unused)]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for E164 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for E164 {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Strip all non-digit characters
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();

        let normalized = match digits.len() {
            // Handle international format with country code
            11 if digits.starts_with('1') => format!("+{}", digits),

            // Handle 10-digit US/Canada numbers
            10 => format!("+1{}", digits),

            // Invalid length
            _ => bail!("Phone number must be 10 digits (or 11 digits starting with 1)"),
        };

        Ok(E164(normalized))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e164_parsing() {
        // Test various formats
        assert_eq!(
            E164::from_str("123-456-7890").unwrap().as_str(),
            "+11234567890"
        );
        assert_eq!(
            E164::from_str("(123) 456-7890").unwrap().as_str(),
            "+11234567890"
        );
        assert_eq!(
            E164::from_str("1234567890").unwrap().as_str(),
            "+11234567890"
        );
        assert_eq!(
            E164::from_str("11234567890").unwrap().as_str(),
            "+11234567890"
        );

        // Test with international format
        assert_eq!(
            E164::from_str("+1-123-456-7890").unwrap().as_str(),
            "+11234567890"
        );

        // Test invalid numbers
        assert!(E164::from_str("123456").is_err());
        assert!(E164::from_str("123456789012").is_err());
        assert!(E164::from_str("abcd").is_err());
    }

    #[test]
    fn test_area_code() {
        let number = E164::from_str("123-456-7890").unwrap();
        assert_eq!(number.area_code(), "123");
    }
}

pub fn format_contact_list(contacts: &[Contact], offset: usize) -> String {
    contacts
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let area_code = E164::from_str(&c.contact_user_number)
                .map(|e| e.area_code().to_string())
                .unwrap_or_else(|_| "???".to_string());
            format!("{}. {} ({})", i + offset + 1, c.contact_name, area_code)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
// In util.rs - replace the entire tests module and selection parsing code
// Keep original E164 code and tests at the top, then add:

pub struct Selection {
    pub index: usize,
    pub sub_selection: Option<char>,
}
pub fn parse_selections(input: &str) -> anyhow::Result<Vec<Selection>> {
    // If input consists of only letters, treat it as invalid format
    if input.chars().all(|c| c.is_alphabetic()) {
        anyhow::bail!("Invalid selection format: {}", input);
    }

    input
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            let s = s.to_lowercase();
            let chars: Vec<char> = s.chars().collect();

            // Check if last character is a letter
            let (num_str, letter) = if let Some(&last_char) = chars.last() {
                if last_char.is_ascii_lowercase() {
                    // Split into number and letter
                    let slice = &s[..s.len() - 1];
                    (slice, Some(last_char))
                } else {
                    (s.as_str(), None)
                }
            } else {
                (s.as_str(), None)
            };

            // Parse the number part
            let num = num_str
                .trim()
                .parse::<usize>()
                .map_err(|_| anyhow::anyhow!("Invalid selection format: {}", s))?;

            if num == 0 {
                anyhow::bail!("Selection numbers must be greater than 0");
            }

            Ok(Selection {
                index: num - 1,
                sub_selection: letter,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests2 {
    use super::*;

    #[test]
    fn test_e164_parsing() {
        // Test various formats
        assert_eq!(
            E164::from_str("123-456-7890").unwrap().as_str(),
            "+11234567890"
        );
        assert_eq!(
            E164::from_str("(123) 456-7890").unwrap().as_str(),
            "+11234567890"
        );
        assert_eq!(
            E164::from_str("1234567890").unwrap().as_str(),
            "+11234567890"
        );
        assert_eq!(
            E164::from_str("11234567890").unwrap().as_str(),
            "+11234567890"
        );

        // Test with international format
        assert_eq!(
            E164::from_str("+1-123-456-7890").unwrap().as_str(),
            "+11234567890"
        );

        // Test invalid numbers
        assert!(E164::from_str("123456").is_err());
        assert!(E164::from_str("123456789012").is_err());
        assert!(E164::from_str("abcd").is_err());
    }

    #[test]
    fn test_area_code() {
        let number = E164::from_str("123-456-7890").unwrap();
        assert_eq!(number.area_code(), "123");
    }

    #[test]
    fn test_parse_selections() -> anyhow::Result<()> {
        // Test basic numeric selections
        let selections = parse_selections("1, 2, 3")?;
        assert_eq!(selections.len(), 3);
        assert_eq!(selections[0].index, 0);
        assert_eq!(selections[0].sub_selection, None);

        // Test selections with letters
        let selections = parse_selections("1b, 2a")?;
        assert_eq!(selections.len(), 2);
        assert_eq!(selections[0].index, 0);
        assert_eq!(selections[0].sub_selection, Some('b'));
        assert_eq!(selections[1].index, 1);
        assert_eq!(selections[1].sub_selection, Some('a'));

        // Test with spaces
        let selections = parse_selections(" 1b , 2a ")?;
        assert_eq!(selections.len(), 2);
        assert_eq!(selections[0].sub_selection, Some('b'));
        assert_eq!(selections[1].sub_selection, Some('a'));

        Ok(())
    }
}
pub struct ResponseBuilder {
    sections: Vec<String>,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    pub fn add_section(&mut self, content: &str) -> &mut Self {
        if !content.is_empty() {
            self.sections.push(format!("\n{content}"));
        }
        self
    }

    pub fn add_titled(&mut self, title: &str, content: String) -> &mut Self {
        if !content.is_empty() {
            if !self.sections.is_empty() {
                self.sections.push("\n".to_string());
            }
            self.sections.push(format!("{}:\n{}", title, content));
        }
        self
    }

    pub fn add_errors(&mut self, errors: &[String]) -> &mut Self {
        if !errors.is_empty() {
            self.add_titled("Errors", errors.join("\n"));
        }
        self
    }

    pub fn build(self) -> String {
        self.sections.join("")
    }
}
