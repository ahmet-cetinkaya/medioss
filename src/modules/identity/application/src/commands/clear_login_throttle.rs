use crate::{ports::LoginThrottleRepository, ApplicationError};

pub async fn clear_login_throttle(
    repository: &impl LoginThrottleRepository,
    identifier: &str,
) -> Result<(), ApplicationError> {
    repository
        .clear_failed_attempts(&identifier.trim().to_lowercase())
        .await
        .map_err(ApplicationError::from)
}

#[cfg(test)]
#[path = "clear_login_throttle.tests.rs"]
mod tests;
