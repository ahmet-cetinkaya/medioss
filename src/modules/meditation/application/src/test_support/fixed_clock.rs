use chrono::{DateTime, Utc};

use crate::ports::Clock;

/// A clock returning a fixed instant.
#[derive(Clone)]
pub struct FixedClock(pub DateTime<Utc>);

#[async_trait::async_trait]
impl Clock for FixedClock {
    async fn now(&self) -> DateTime<Utc> {
        self.0
    }
}
