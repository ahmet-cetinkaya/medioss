use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};

use crate::IdentityError;

const MAX_DELAY_EXPONENT: u64 = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedAttempts {
    count: u64,
    last_attempt_at: DateTime<Utc>,
    blocked_until: DateTime<Utc>,
}

impl FailedAttempts {
    pub fn restore(
        count: u64,
        last_attempt_at: DateTime<Utc>,
        blocked_until: DateTime<Utc>,
    ) -> Result<Self, IdentityError> {
        if count == 0 || blocked_until <= last_attempt_at {
            return Err(IdentityError::InvalidThrottleState);
        }

        Ok(Self {
            count,
            last_attempt_at,
            blocked_until,
        })
    }

    pub fn first_failure(attempted_at: DateTime<Utc>) -> Self {
        Self::next_failure(1, attempted_at)
    }

    pub fn record_failure(&self, attempted_at: DateTime<Utc>) -> Self {
        if self.is_blocked_at(attempted_at) {
            return self.clone();
        }

        Self::next_failure(self.count.saturating_add(1), attempted_at)
    }

    fn next_failure(count: u64, attempted_at: DateTime<Utc>) -> Self {
        let delay = TimeDelta::seconds(required_delay(count).as_secs() as i64);

        Self {
            count,
            last_attempt_at: attempted_at,
            blocked_until: attempted_at + delay,
        }
    }

    pub const fn count(&self) -> u64 {
        self.count
    }

    pub const fn last_attempt_at(&self) -> DateTime<Utc> {
        self.last_attempt_at
    }

    pub const fn blocked_until(&self) -> DateTime<Utc> {
        self.blocked_until
    }

    pub fn retry_after(&self, now: DateTime<Utc>) -> Option<Duration> {
        self.blocked_until
            .signed_duration_since(now)
            .to_std()
            .ok()
            .filter(|remaining| !remaining.is_zero())
    }

    fn is_blocked_at(&self, now: DateTime<Utc>) -> bool {
        self.blocked_until > now
    }
}

pub fn required_delay(failure_count: u64) -> Duration {
    let exponent = failure_count.saturating_sub(1).min(MAX_DELAY_EXPONENT) as u32;
    Duration::from_secs(1_u64 << exponent)
}

#[cfg(test)]
#[path = "failed_attempts.tests.rs"]
mod tests;
