use std::sync::Arc;

use chrono::{DateTime, Duration, TimeZone, Utc};
use kernel::UserId;

use crate::ports::SessionRepository;
use crate::queries::get_user_statistics::{
    GetUserStatistics, GetUserStatisticsHandler, GetUserStatisticsService, StatisticsDay,
    StatisticsRange,
};
use crate::test_support::{user_a, FakeSessionRepository, FixedClock, RepoState};

fn at(day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, day, hour, minute, 0)
        .single()
        .expect("test instant")
}

async fn record(
    sessions: &FakeSessionRepository,
    user_id: UserId,
    meditation_id: i64,
    started_at: DateTime<Utc>,
    minutes: i64,
) {
    sessions
        .record_if_new(
            user_id,
            meditation_id,
            started_at,
            started_at + Duration::minutes(minutes),
        )
        .await
        .expect("session");
}

fn new_sessions() -> Arc<FakeSessionRepository> {
    Arc::new(FakeSessionRepository(RepoState::new()))
}

async fn statistics(
    sessions: &Arc<FakeSessionRepository>,
    user_id: UserId,
    range: StatisticsRange,
    tz_offset_minutes: i16,
    now: DateTime<Utc>,
) -> crate::queries::get_user_statistics::UserStatistics {
    GetUserStatisticsService::new(sessions.clone(), Arc::new(FixedClock(now)))
        .handle(GetUserStatistics {
            user_id,
            range,
            tz_offset_minutes,
        })
        .await
        .expect("statistics")
}

#[tokio::test]
async fn user_statistics_aggregate_days_ranges_and_all_time_streaks() {
    let sessions = new_sessions();
    let user = user_a();
    record(
        &sessions,
        user,
        99,
        Utc.with_ymd_and_hms(2026, 8, 6, 9, 0, 0)
            .single()
            .expect("test instant"),
        40,
    )
    .await;
    record(&sessions, user, 1, at(1, 9, 0), 25).await;
    record(&sessions, user, 2, at(1, 10, 0), 35).await;
    record(&sessions, user, 3, at(2, 9, 0), 10).await;
    record(&sessions, user, 4, at(3, 9, 0), 10).await;
    record(&sessions, user, 5, at(4, 9, 0), 10).await;
    record(&sessions, user, 6, at(5, 9, 0), 10).await;
    record(&sessions, user, 7, at(7, 9, 0), 50).await;

    let now = at(6, 12, 0);
    let all = statistics(&sessions, user, StatisticsRange::AllTime, 0, now).await;
    assert_eq!(all.total_minutes, 140);
    assert_eq!(all.total_days, 6);
    assert_eq!(all.current_streak, 5);
    assert_eq!(all.longest_streak, 5);
    assert_eq!(
        all.calendar[0],
        StatisticsDay {
            date: Utc
                .with_ymd_and_hms(2026, 8, 6, 0, 0, 0)
                .single()
                .expect("test instant")
                .date_naive(),
            minutes: 40,
        }
    );

    let last_30 = statistics(&sessions, user, StatisticsRange::Last30Days, 0, now).await;
    assert_eq!(last_30.total_minutes, 100);
    assert_eq!(last_30.total_days, 5);
    assert_eq!(last_30.current_streak, 5);

    let today = statistics(&sessions, user, StatisticsRange::Today, 0, now).await;
    assert_eq!(today.total_minutes, 0);
    assert_eq!(today.total_days, 0);
    assert!(today.calendar.is_empty());
    assert!(!last_30
        .calendar
        .iter()
        .any(|day| day.date == at(7, 0, 0).date_naive()));
}

#[tokio::test]
async fn user_statistics_isolated_by_user_and_uses_local_start_day() {
    let sessions = new_sessions();
    let user = user_a();
    record(&sessions, user, 1, at(1, 22, 30), 90).await;
    record(&sessions, user, 3, at(3, 9, 0), 0).await;
    record(&sessions, UserId::new(2), 2, at(2, 9, 0), 60).await;

    let statistics_for_user =
        statistics(&sessions, user, StatisticsRange::AllTime, 180, at(3, 12, 0)).await;
    assert_eq!(statistics_for_user.total_minutes, 90);
    assert_eq!(statistics_for_user.total_days, 2);
    assert_eq!(
        statistics_for_user.calendar,
        vec![
            StatisticsDay {
                date: at(2, 0, 0).date_naive(),
                minutes: 90
            },
            StatisticsDay {
                date: at(3, 0, 0).date_naive(),
                minutes: 0
            },
        ]
    );

    let empty = statistics(
        &sessions,
        UserId::new(3),
        StatisticsRange::AllTime,
        0,
        at(3, 12, 0),
    )
    .await;
    assert_eq!(empty.total_minutes, 0);
    assert_eq!(empty.total_days, 0);
    assert!(empty.calendar.is_empty());
}

#[tokio::test]
async fn custom_statistics_range_counts_both_end_dates_but_not_neighbours() {
    let sessions = new_sessions();
    let user = user_a();
    record(&sessions, user, 1, at(3, 9, 0), 10).await;
    record(&sessions, user, 2, at(4, 9, 0), 20).await;
    record(&sessions, user, 3, at(5, 9, 0), 30).await;
    record(&sessions, user, 4, at(6, 9, 0), 40).await;
    let now = at(6, 12, 0);
    let range =
        StatisticsRange::custom(at(4, 0, 0).date_naive(), at(5, 0, 0).date_naive()).expect("range");

    let stats = statistics(&sessions, user, range, 0, now).await;

    assert_eq!(stats.total_minutes, 50, "20 + 30, neither neighbour");
    assert_eq!(stats.total_days, 2);
}

#[tokio::test]
async fn custom_statistics_range_never_reaches_past_today() {
    let sessions = new_sessions();
    let user = user_a();
    record(&sessions, user, 1, at(6, 9, 0), 15).await;
    record(&sessions, user, 2, at(7, 9, 0), 60).await;
    let now = at(6, 12, 0);
    let range =
        StatisticsRange::custom(at(6, 0, 0).date_naive(), at(7, 0, 0).date_naive()).expect("range");

    let stats = statistics(&sessions, user, range, 0, now).await;

    assert_eq!(stats.total_minutes, 15, "a future session is excluded");
}
