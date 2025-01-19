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
