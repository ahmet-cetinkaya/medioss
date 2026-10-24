use super::list_sounds_query::ListSoundsQuery;
use crate::ApplicationError;

/// These queries carry no input to check, so every query is accepted.
#[derive(Debug, Default, Clone, Copy)]
pub struct ListSoundsQueryValidator;

impl ListSoundsQueryValidator {
    pub fn validate(&self, _query: &ListSoundsQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "list_sounds_query_validator.tests.rs"]
mod tests;
