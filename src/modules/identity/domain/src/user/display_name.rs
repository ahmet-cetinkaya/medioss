use std::fmt;

use super::IdentityError;

const MAX_DISPLAY_NAME_LENGTH: usize = 64;

#[derive(Clone, PartialEq, Eq)]
pub struct DisplayName(String);

impl DisplayName {
    pub fn parse(value: impl Into<String>) -> Result<Self, IdentityError> {
        let trimmed = value.into().trim().to_owned();

        if trimmed.is_empty() {
            return Err(IdentityError::InvalidProfile);
        }

        if trimmed.chars().count() > MAX_DISPLAY_NAME_LENGTH {
            return Err(IdentityError::InvalidProfile);
        }

        Ok(Self(trimmed))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for DisplayName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("DisplayName({:?})", self.0))
    }
}

#[cfg(test)]
#[path = "display_name.tests.rs"]
mod tests;
