use chrono::FixedOffset;

use super::get_timeline_query::GetTimelineQuery;
use crate::ApplicationError;

const MIN_PAGE_SIZE: u16 = 1;
const MAX_PAGE_SIZE: u16 = 100;

/// Input rules for reading the timeline: a page size from 1 to 100 and a
/// timezone offset that is a real UTC offset.
#[derive(Debug, Default, Clone, Copy)]
pub struct GetTimelineQueryValidator;

impl GetTimelineQueryValidator {
    pub fn validate(&self, query: &GetTimelineQuery) -> Result<(), ApplicationError> {
        if !(MIN_PAGE_SIZE..=MAX_PAGE_SIZE).contains(&query.limit) {
            return Err(ApplicationError::InvalidInput(
                "page size must be between 1 and 100",
            ));
        }
        self.local_offset(query)?;
        Ok(())
    }

    pub fn local_offset(&self, query: &GetTimelineQuery) -> Result<FixedOffset, ApplicationError> {
        FixedOffset::east_opt(i32::from(query.tz_offset_minutes) * 60)
            .ok_or(ApplicationError::InvalidInput("invalid timezone offset"))
    }
}

#[cfg(test)]
#[path = "get_timeline_query_validator.tests.rs"]
mod tests;
