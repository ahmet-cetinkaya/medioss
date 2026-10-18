use std::sync::Arc;

use chrono::{Duration, TimeZone, Utc};
use kernel::UserId;

use super::*;
use crate::commands::{create_program, upload_meditation};
use crate::ports::{
    CategoryRepository, MeditationRepository, MembershipRepository, PlaybackStateRepository,
    ProgramRepository, SessionRepository,
};
use crate::test_support::{
    FakeBlobStore, FakeCategoryRepository, FakeConverter, FakeCoverValidator,
    FakeMeditationRepository, FakeMembershipRepository, FakePlaybackRepository,
    FakeProgramRepository, FakeSessionRepository, FakeValidator, FixedClock, RepoState,
};

fn user_a() -> UserId {
    UserId::new(1)
}

fn at(day: u32, hour: u32, minute: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, day, hour, minute, 0)
        .single()
        .expect("test instant")
}

async fn record(
    sessions: &FakeSessionRepository,
    user_id: UserId,
    meditation_id: i64,
    started_at: chrono::DateTime<Utc>,
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

struct Deps {
    meditations: FakeMeditationRepository,
    programs: FakeProgramRepository,
    memberships: FakeMembershipRepository,
    playback: FakePlaybackRepository,
    store: FakeBlobStore,
    clock: FixedClock,
}

impl Deps {
    fn new() -> Self {
        let state = RepoState::new();
        Self {
            meditations: FakeMeditationRepository(state.clone()),
            programs: FakeProgramRepository(state.clone()),
            memberships: FakeMembershipRepository(state.clone()),
            playback: FakePlaybackRepository(state),
            store: FakeBlobStore::new(),
            clock: FixedClock(Utc::now()),
        }
    }

    async fn seed_program(&self, title: &str) -> i64 {
        create_program(
            &self.programs,
            &FakeCoverValidator,
            &self.store,
            &self.clock,
            title,
            None,
            None,
        )
        .await
        .expect("program")
        .id
    }

    async fn seed_meditation_in(
        &self,
        program_id: i64,
        name: &str,
    ) -> meditation_domain::Meditation {
        upload_meditation(
            &self.meditations,
            &self.memberships,
            &self.programs,
            &FakeValidator,
            &FakeConverter { fail: false },
            &self.store,
            &self.clock,
            name,
            program_id,
            b"OggS....OpusHead".to_vec(),
            None,
        )
        .await
        .expect("upload")
    }
}

#[tokio::test]
async fn continue_program_resolves_the_home_program_of_the_last_touch() {
    let deps = Deps::new();
    let program = deps.seed_program("Morning").await;
    let meditation = deps.seed_meditation_in(program, "M1").await;
    deps.playback
        .upsert_position(user_a(), meditation.id(), 1_000)
        .await
        .expect("touch");

    let found = get_continue_program(&deps.playback, &deps.memberships, &deps.programs, user_a())
        .await
        .expect("query")
        .expect("some");
    assert_eq!(found.program.id, program);
    assert_eq!(found.meditation_id, meditation.id());
}

#[tokio::test]
async fn continue_program_is_none_for_an_untouched_user() {
    let deps = Deps::new();
    let found = get_continue_program(&deps.playback, &deps.memberships, &deps.programs, user_a())
        .await
        .expect("query");
    assert!(found.is_none());
}

#[tokio::test]
async fn continue_program_falls_back_past_a_deleted_meditations_touch() {
    // The most recently touched meditation has no membership at all (as if
    // its program had been deleted and the meditation later removed too,
    // or the membership row otherwise vanished); the query must not give up
    // and must resolve to the next most-recently-touched live one instead.
    let deps = Deps::new();
    let program = deps.seed_program("Evening").await;
    let live = deps.seed_meditation_in(program, "Live").await;
    deps.playback
        .upsert_position(user_a(), live.id(), 500)
        .await
        .expect("touch live");
    // A touch on a meditation id with no membership row anywhere.
    let orphan_id = live.id() + 999;
    deps.playback
        .upsert_position(user_a(), orphan_id, 900)
        .await
        .expect("touch orphan");

    let found = get_continue_program(&deps.playback, &deps.memberships, &deps.programs, user_a())
        .await
        .expect("query")
        .expect("some");
    assert_eq!(found.meditation_id, live.id());
}

#[tokio::test]
async fn user_statistics_aggregate_days_ranges_and_all_time_streaks() {
    let state = RepoState::new();
    let sessions = FakeSessionRepository(state);
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
    let all = get_user_statistics(&sessions, user, StatisticsRange::AllTime, 0, now)
        .await
        .expect("all statistics");
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

    let last_30 = get_user_statistics(&sessions, user, StatisticsRange::Last30Days, 0, now)
        .await
        .expect("last 30 statistics");
    assert_eq!(last_30.total_minutes, 100);
    assert_eq!(last_30.total_days, 5);
    assert_eq!(last_30.current_streak, 5);

    let today = get_user_statistics(&sessions, user, StatisticsRange::Today, 0, now)
        .await
        .expect("today statistics");
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
    let state = RepoState::new();
    let sessions = FakeSessionRepository(state);
    let user = user_a();
    record(&sessions, user, 1, at(1, 22, 30), 90).await;
    record(&sessions, user, 3, at(3, 9, 0), 0).await;
    record(&sessions, UserId::new(2), 2, at(2, 9, 0), 60).await;

    let statistics =
        get_user_statistics(&sessions, user, StatisticsRange::AllTime, 180, at(3, 12, 0))
            .await
            .expect("statistics");
    assert_eq!(statistics.total_minutes, 90);
    assert_eq!(statistics.total_days, 2);
    assert_eq!(
        statistics.calendar,
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

    let empty = get_user_statistics(
        &sessions,
        UserId::new(3),
        StatisticsRange::AllTime,
        0,
        at(3, 12, 0),
    )
    .await
    .expect("empty statistics");
    assert_eq!(empty.total_minutes, 0);
    assert_eq!(empty.total_days, 0);
    assert!(empty.calendar.is_empty());
}

#[tokio::test]
async fn daily_suggestion_is_the_same_pick_for_repeated_calls_same_day() {
    let deps = Deps::new();
    let program = deps.seed_program("Library Home").await;
    deps.seed_meditation_in(program, "A").await;
    deps.seed_meditation_in(program, "B").await;
    deps.seed_meditation_in(program, "C").await;

    let first = get_daily_suggestion(
        &deps.meditations,
        &deps.clock,
        &deps.memberships,
        &deps.programs,
    )
    .await
    .expect("query")
    .expect("some");
    let second = get_daily_suggestion(
        &deps.meditations,
        &deps.clock,
        &deps.memberships,
        &deps.programs,
    )
    .await
    .expect("query")
    .expect("some");
    assert_eq!(first.meditation.id(), second.meditation.id());
    assert!(first.program.is_some());
}

#[tokio::test]
async fn daily_suggestion_is_none_for_an_empty_library() {
    let deps = Deps::new();
    let found = get_daily_suggestion(
        &deps.meditations,
        &deps.clock,
        &deps.memberships,
        &deps.programs,
    )
    .await
    .expect("query");
    assert!(found.is_none());
}

#[tokio::test]
async fn recent_programs_dedupes_by_program_most_recent_first_and_honors_limit() {
    let deps = Deps::new();
    let program_a = deps.seed_program("A").await;
    let program_b = deps.seed_program("B").await;
    let med_a1 = deps.seed_meditation_in(program_a, "A1").await;
    let med_a2 = deps.seed_meditation_in(program_a, "A2").await;
    let med_b = deps.seed_meditation_in(program_b, "B1").await;

    // Touch order (oldest to newest): A1, B1, A2. A2's touch should keep A
    // as the most-recent program (A2 is newer than B1's touch of B), and
    // A1's earlier touch of the same program must not appear twice.
    deps.playback
        .upsert_position(user_a(), med_a1.id(), 100)
        .await
        .expect("touch a1");
    deps.playback
        .upsert_position(user_a(), med_b.id(), 100)
        .await
        .expect("touch b1");
    deps.playback
        .upsert_position(user_a(), med_a2.id(), 100)
        .await
        .expect("touch a2");

    let all = list_recent_programs(
        &deps.playback,
        &deps.memberships,
        &deps.programs,
        user_a(),
        None,
    )
    .await
    .expect("query");
    assert_eq!(all.len(), 2, "each program appears once");
    assert_eq!(
        all[0].program.id, program_a,
        "most recently touched program first"
    );
    assert_eq!(all[1].program.id, program_b);

    let limited = list_recent_programs(
        &deps.playback,
        &deps.memberships,
        &deps.programs,
        user_a(),
        Some(1),
    )
    .await
    .expect("query");
    assert_eq!(limited.len(), 1);
    assert_eq!(limited[0].program.id, program_a);
}

async fn seeded_list_library() -> (
    Arc<FakeMeditationRepository>,
    Arc<FakeMembershipRepository>,
    Arc<FakeProgramRepository>,
    Arc<FakeCategoryRepository>,
) {
    let meditations = Arc::new(FakeMeditationRepository(RepoState::new()));
    let memberships = Arc::new(FakeMembershipRepository(RepoState::new()));
    let programs = Arc::new(FakeProgramRepository(RepoState::new()));
    let categories = Arc::new(FakeCategoryRepository(RepoState::new()));
    let now = Utc::now();

    for i in 0..120 {
        let name = match i % 6 {
            0 => "Morning Breathing",
            1 => "Café Quietude",
            2 => "Evening Calm",
            3 => "Dreamscape",
            4 => "Breathing Exercise",
            _ => "Forest Walk",
        };
        meditations
            .insert(
                name.to_owned(),
                "audio/ogg",
                1000,
                format!("k/{name}/{i}"),
                1000,
                now,
                None,
            )
            .await
            .unwrap();
    }
    let focus_program = programs
        .insert("Focus Program".to_owned(), None, now)
        .await
        .unwrap()
        .id;
    let sleep = categories.insert("Sleep".to_owned(), now).await.unwrap();
    let home_program = programs
        .insert("Home Collection".to_owned(), None, now)
        .await
        .unwrap()
        .id;
    programs
        .set_category(home_program, Some(sleep.id))
        .await
        .unwrap();
    memberships.add(focus_program, 3, now).await.unwrap();
    memberships.add(home_program, 5, now).await.unwrap();
    (meditations, memberships, programs, categories)
}

async fn search_library(
    seeded: &(
        Arc<FakeMeditationRepository>,
        Arc<FakeMembershipRepository>,
        Arc<FakeProgramRepository>,
        Arc<FakeCategoryRepository>,
    ),
    query: Option<&str>,
) -> Vec<i64> {
    list_library(
        seeded.0.as_ref(),
        seeded.1.as_ref(),
        seeded.2.as_ref(),
        seeded.3.as_ref(),
        query,
    )
    .await
    .unwrap()
    .iter()
    .map(|meditation| meditation.id())
    .collect()
}

#[tokio::test]
async fn blank_query_returns_the_full_library() {
    let seeded = seeded_list_library().await;
    for query in [None, Some(""), Some("   ")] {
        let ids = search_library(&seeded, query).await;
        assert_eq!(ids.len(), 120, "query: {query:?}");
    }
}

#[tokio::test]
async fn name_matches_case_and_diacritic_insensitively() {
    let seeded = seeded_list_library().await;
    for query in ["morning", "MORNING", "breath"] {
        let ids = search_library(&seeded, Some(query)).await;
        let expected = if query == "breath" { 40 } else { 20 };
        assert_eq!(ids.len(), expected, "query: {query}");
    }
    let ids = search_library(&seeded, Some("cafe quietude")).await;
    assert_eq!(ids.len(), 20);
}

#[tokio::test]
async fn program_and_category_names_match() {
    let seeded = seeded_list_library().await;
    let ids = search_library(&seeded, Some("focus")).await;
    assert_eq!(ids, vec![3]);
    let ids = search_library(&seeded, Some("sleep")).await;
    assert_eq!(ids, vec![5]);
}

#[tokio::test]
async fn multi_term_requires_every_term() {
    let seeded = seeded_list_library().await;
    let ids = search_library(&seeded, Some("morning breathing")).await;
    assert_eq!(ids.len(), 20);
    let ids = search_library(&seeded, Some("morning dreamscape")).await;
    assert!(ids.is_empty());
}

#[tokio::test]
async fn no_match_returns_an_empty_list() {
    let seeded = seeded_list_library().await;
    let ids = search_library(&seeded, Some("xyzzy")).await;
    assert!(ids.is_empty());
}

fn local_day(month: u32, day: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(2026, month, day).expect("test date")
}

#[test]
fn custom_statistics_range_accepts_valid_spans_and_rejects_bad_ones() {
    let from = local_day(9, 1);
    assert_eq!(
        StatisticsRange::custom(from, from).expect("one day"),
        StatisticsRange::Custom { from, to: from }
    );
    let last_valid = from + Duration::days(MAX_CUSTOM_RANGE_DAYS - 1);
    assert!(StatisticsRange::custom(from, last_valid).is_ok());
    assert!(StatisticsRange::custom(from, last_valid + Duration::days(1)).is_err());
    assert!(StatisticsRange::custom(local_day(9, 2), from).is_err());
}

#[tokio::test]
async fn custom_statistics_range_counts_both_end_dates_but_not_neighbours() {
    let state = RepoState::new();
    let sessions = FakeSessionRepository(state);
    let user = user_a();
    record(&sessions, user, 1, at(3, 9, 0), 10).await;
    record(&sessions, user, 2, at(4, 9, 0), 20).await;
    record(&sessions, user, 3, at(5, 9, 0), 30).await;
    record(&sessions, user, 4, at(6, 9, 0), 40).await;
    let now = at(6, 12, 0);
    let range =
        StatisticsRange::custom(at(4, 0, 0).date_naive(), at(5, 0, 0).date_naive()).expect("range");

    let stats = get_user_statistics(&sessions, user, range, 0, now)
        .await
        .expect("custom statistics");

    assert_eq!(stats.total_minutes, 50, "20 + 30, neither neighbour");
    assert_eq!(stats.total_days, 2);
}

#[tokio::test]
async fn custom_statistics_range_never_reaches_past_today() {
    let state = RepoState::new();
    let sessions = FakeSessionRepository(state);
    let user = user_a();
    record(&sessions, user, 1, at(6, 9, 0), 15).await;
    record(&sessions, user, 2, at(7, 9, 0), 60).await;
    let now = at(6, 12, 0);
    let range =
        StatisticsRange::custom(at(6, 0, 0).date_naive(), at(7, 0, 0).date_naive()).expect("range");

    let stats = get_user_statistics(&sessions, user, range, 0, now)
        .await
        .expect("custom statistics");

    assert_eq!(stats.total_minutes, 15, "a future session is excluded");
}
