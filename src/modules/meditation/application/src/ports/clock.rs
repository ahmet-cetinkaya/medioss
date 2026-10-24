use async_trait::async_trait;

/// Tells the current time, so use-cases stay testable.
#[async_trait]
pub trait Clock: Send + Sync {
    /// The current wall-clock time.
    async fn now(&self) -> chrono::DateTime<chrono::Utc>;
}
