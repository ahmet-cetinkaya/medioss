use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait Clock: Send + Sync {
    async fn now(&self) -> DateTime<Utc>;
}
