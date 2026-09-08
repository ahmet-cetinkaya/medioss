use std::fmt;

use super::IdentityError;

const MIN_PASSWORD_LENGTH: usize = 8;

#[derive(Clone, PartialEq, Eq)]
pub struct RawPassword(String);

impl RawPassword {
    pub fn parse(value: impl Into<String>) -> Result<Self, IdentityError> {
        let value = value.into();

        if value.chars().count() < MIN_PASSWORD_LENGTH {
            return Err(IdentityError::WeakPassword);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for RawPassword {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RawPassword([REDACTED])")
    }
}

#[cfg(test)]
#[path = "raw_password.tests.rs"]
mod tests;
