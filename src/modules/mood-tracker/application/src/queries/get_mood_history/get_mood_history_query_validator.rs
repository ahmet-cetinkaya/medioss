use chrono::FixedOffset;

use super::get_mood_history_query::GetMoodHistoryQuery;
use crate::ApplicationError;

const MIN_PAGE_SIZE: u16 = 1;
const MAX_PAGE_SIZE: u16 = 100;

/// Input rules for reading mood history: a page size from 1 to 100 and a
/// timezone offset that is a real UTC offset.
#[derive(Debug, Default, Clone, Copy)]
pub struct GetMoodHistoryQueryValidator;

impl GetMoodHistoryQueryValidator {
    pub fn validate(&self, query: &GetMoodHistoryQuery) -> Result<(), ApplicationError> {
        if !(MIN_PAGE_SIZE..=MAX_PAGE_SIZE).contains(&query.limit) {
            return Err(ApplicationError::InvalidInput(
                "page size must be between 1 and 100",
            ));
        }
        self.local_offset(query)?;
        Ok(())
    }

    pub fn local_offset(
        &self,
        query: &GetMoodHistoryQuery,
    ) -> Result<FixedOffset, ApplicationError> {
        FixedOffset::east_opt(i32::from(query.tz_offset_minutes) * 60)
            .ok_or(ApplicationError::InvalidInput("invalid timezone offset"))
    }
}

#[cfg(test)]
#[path = "get_mood_history_query_validator.tests.rs"]
mod tests;
