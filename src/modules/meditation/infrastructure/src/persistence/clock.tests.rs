use chrono::Utc;
use meditation_application::ports::Clock;

use super::SystemClock;

#[tokio::test]
async fn now_lies_between_surrounding_system_readings() {
    let before = Utc::now();
    let reading = SystemClock.now().await;
    let after = Utc::now();

    assert!(before <= reading);
    assert!(reading <= after);
}

#[tokio::test]
async fn now_is_monotonic_across_successive_calls() {
    let clock = SystemClock;
    let first = clock.now().await;
    let second = clock.now().await;

    assert!(second >= first);
}
