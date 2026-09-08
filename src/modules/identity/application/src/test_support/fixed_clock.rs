use chrono::{DateTime, TimeZone, Utc};

use crate::ports::Clock;

pub(crate) struct FixedClock {
    now: DateTime<Utc>,
}

impl FixedClock {
    pub(crate) fn new() -> Self {
        Self {
            now: Utc.with_ymd_and_hms(2026, 9, 8, 10, 0, 0).unwrap(),
        }
    }

    pub(crate) fn now_value(&self) -> DateTime<Utc> {
        self.now
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.now
    }
}
