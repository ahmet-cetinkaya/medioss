use async_trait::async_trait;
use chrono::{DateTime, Utc};
use identity_domain::FailedAttempts;

use super::RepositoryError;

#[async_trait]
pub trait LoginThrottleRepository: Send + Sync {
    async fn failed_attempts(
        &self,
        normalized_identifier: &str,
    ) -> Result<Option<FailedAttempts>, RepositoryError>;

    async fn record_failed_attempt(
        &self,
        normalized_identifier: &str,
        attempted_at: DateTime<Utc>,
    ) -> Result<FailedAttempts, RepositoryError>;

    async fn clear_failed_attempts(
        &self,
        normalized_identifier: &str,
    ) -> Result<(), RepositoryError>;
}
