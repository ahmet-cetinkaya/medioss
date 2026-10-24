use super::get_playback_state_query::GetPlaybackStateQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct GetPlaybackStateQueryValidator;

impl GetPlaybackStateQueryValidator {
    pub fn validate(&self, _query: &GetPlaybackStateQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "get_playback_state_query_validator.tests.rs"]
mod tests;
