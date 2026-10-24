use chrono::FixedOffset;

use super::get_mood_statistics_query::GetMoodStatisticsQuery;
use crate::ApplicationError;

/// Input rule for mood statistics: the timezone offset is a real UTC offset.
#[derive(Debug, Default, Clone, Copy)]
pub struct GetMoodStatisticsQueryValidator;

impl GetMoodStatisticsQueryValidator {
    pub fn validate(&self, query: &GetMoodStatisticsQuery) -> Result<(), ApplicationError> {
        self.local_offset(query)?;
        Ok(())
    }

    pub fn local_offset(
        &self,
        query: &GetMoodStatisticsQuery,
    ) -> Result<FixedOffset, ApplicationError> {
        FixedOffset::east_opt(i32::from(query.tz_offset_minutes) * 60)
            .ok_or(ApplicationError::InvalidInput("invalid timezone offset"))
    }
}

#[cfg(test)]
#[path = "get_mood_statistics_query_validator.tests.rs"]
mod tests;
