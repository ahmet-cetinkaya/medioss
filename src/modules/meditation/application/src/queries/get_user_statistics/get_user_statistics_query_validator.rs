use chrono::FixedOffset;

use super::get_user_statistics_query::GetUserStatisticsQuery;
use crate::ApplicationError;

const SECONDS_PER_MINUTE: i32 = 60;

#[derive(Debug, Default, Clone, Copy)]
pub struct GetUserStatisticsQueryValidator;

impl GetUserStatisticsQueryValidator {
    pub fn validate(&self, query: &GetUserStatisticsQuery) -> Result<(), ApplicationError> {
        self.timezone_offset(query.tz_offset_minutes).map(|_| ())
    }

    /// The caller's UTC offset, rejected when it is not a real timezone offset.
    pub fn timezone_offset(&self, tz_offset_minutes: i16) -> Result<FixedOffset, ApplicationError> {
        FixedOffset::east_opt(i32::from(tz_offset_minutes) * SECONDS_PER_MINUTE)
            .ok_or(ApplicationError::InvalidInput("invalid timezone offset"))
    }
}

#[cfg(test)]
#[path = "get_user_statistics_query_validator.tests.rs"]
mod tests;
