use std::fmt;

use serde::{de, Deserialize, Deserializer, Serialize};

use super::IdentityError;

const MAX_EMAIL_LENGTH: usize = 254;

#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct Email(String);

impl Email {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, IdentityError> {
        let canonical = value.as_ref().trim().to_lowercase();
        let is_valid = !canonical.is_empty()
            && canonical.contains('@')
            && canonical.chars().count() <= MAX_EMAIL_LENGTH;

        if !is_valid {
            return Err(IdentityError::InvalidEmail);
        }

        Ok(Self(canonical))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Email {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Email([REDACTED])")
    }
}

impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(de::Error::custom)
    }
}

#[cfg(test)]
#[path = "email.tests.rs"]
mod tests;
