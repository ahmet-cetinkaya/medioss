use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::ports::Clock;

pub struct FixedClock(pub DateTime<Utc>);

#[async_trait]
impl Clock for FixedClock {
    async fn now(&self) -> DateTime<Utc> {
        self.0
    }
}
