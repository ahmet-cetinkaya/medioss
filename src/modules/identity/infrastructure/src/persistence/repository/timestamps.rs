use chrono::{DateTime, Utc};
use identity_application::ports::RepositoryError;

pub(super) fn parse_rfc3339(value: &str) -> Result<DateTime<Utc>, RepositoryError> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| RepositoryError::OperationFailed)
}

pub(super) fn to_rfc3339(value: &DateTime<Utc>) -> String {
    value.to_rfc3339()
}

#[cfg(test)]
#[path = "timestamps.tests.rs"]
mod tests;
