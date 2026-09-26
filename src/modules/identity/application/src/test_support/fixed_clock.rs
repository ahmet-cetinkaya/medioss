use std::{sync::Mutex, time::Duration};

use chrono::{DateTime, TimeDelta, TimeZone, Utc};

use crate::ports::Clock;

pub(crate) struct FixedClock {
    now: Mutex<DateTime<Utc>>,
}

impl FixedClock {
    pub(crate) fn new() -> Self {
        Self {
            now: Mutex::new(Utc.with_ymd_and_hms(2026, 9, 8, 10, 0, 0).unwrap()),
        }
    }

    pub(crate) fn now_value(&self) -> DateTime<Utc> {
        *self.now.lock().unwrap()
    }

    pub(crate) fn advance(&self, duration: Duration) {
        let delta = TimeDelta::from_std(duration).unwrap();
        let mut now = self.now.lock().unwrap();
        *now = now.checked_add_signed(delta).unwrap();
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.now_value()
    }
}
