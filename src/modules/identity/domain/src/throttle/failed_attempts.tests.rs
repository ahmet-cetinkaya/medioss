use std::time::Duration;

use chrono::{TimeDelta, TimeZone, Utc};

use super::*;

fn now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 8, 10, 0, 0).unwrap()
}

#[test]
fn first_failure_blocks_for_one_second() {
    let attempts = FailedAttempts::first_failure(now());

    assert_eq!(attempts.count(), 1);
    assert_eq!(attempts.last_attempt_at(), now());
    assert_eq!(attempts.blocked_until(), now() + TimeDelta::seconds(1));
    assert_eq!(attempts.retry_after(now()), Some(Duration::from_secs(1)));
}

#[test]
fn failures_after_the_block_elapses_grow_exponentially_and_cap_at_64_seconds() {
    let mut attempts: Option<FailedAttempts> = None;
    let mut attempted_at = now();
    let expected_delays = [1, 2, 4, 8, 16, 32, 64, 64];

    for expected_delay in expected_delays {
        let next = attempts.as_ref().map_or_else(
            || FailedAttempts::first_failure(attempted_at),
            |current| current.record_failure(attempted_at),
        );
        assert_eq!(
            next.blocked_until(),
            attempted_at + TimeDelta::seconds(expected_delay)
        );
        attempted_at = next.blocked_until();
        attempts = Some(next);
    }

    assert_eq!(attempts.unwrap().count(), expected_delays.len() as u64);
}

#[test]
fn failure_during_an_active_block_does_not_extend_blocked_until() {
    let first = FailedAttempts::first_failure(now());
    let blocked_until = first.blocked_until();

    let ignored = first.record_failure(now() + TimeDelta::milliseconds(500));

    assert_eq!(ignored, first);
    assert_eq!(ignored.blocked_until(), blocked_until);
}

#[test]
fn required_delay_is_capped_without_shift_overflow() {
    assert_eq!(required_delay(1), Duration::from_secs(1));
    assert_eq!(required_delay(7), Duration::from_secs(64));
    assert_eq!(required_delay(100), Duration::from_secs(64));
    assert_eq!(required_delay(u64::MAX), Duration::from_secs(64));
}

#[test]
fn restore_rejects_invalid_persisted_state() {
    assert_eq!(
        FailedAttempts::restore(0, now(), now() + TimeDelta::seconds(1)),
        Err(crate::IdentityError::InvalidThrottleState)
    );
    assert_eq!(
        FailedAttempts::restore(1, now(), now()),
        Err(crate::IdentityError::InvalidThrottleState)
    );
}
