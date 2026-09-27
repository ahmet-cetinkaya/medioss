//! Wall-clock `Clock` implementation.
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use meditation_application::ports::Clock;

/// The real system clock; tests use the application fakes instead.
#[derive(Clone, Copy, Default)]
pub struct SystemClock;

#[async_trait]
impl Clock for SystemClock {
    async fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
