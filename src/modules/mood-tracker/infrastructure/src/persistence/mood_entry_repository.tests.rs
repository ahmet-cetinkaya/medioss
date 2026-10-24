use std::time::Duration as StdDuration;

use chrono::{DateTime, Duration, TimeZone, Utc};
use kernel::UserId;
use mood_tracker_application::ports::MoodEntryRepository;
use mood_tracker_domain::MoodScore;
use sea_orm::{ConnectionTrait, DatabaseConnection, SqlxSqliteConnector, Statement};
use sqlx::sqlite::SqlitePoolOptions;

use super::SeaOrmMoodEntryRepository;
use crate::migrations::migrate;

const ALICE: i64 = 1;
const BOB: i64 = 2;

async fn open() -> DatabaseConnection {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .idle_timeout(None::<StdDuration>)
        .max_lifetime(None::<StdDuration>)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    let connection = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
    migrate(&connection).await.unwrap();
    connection
}

fn noon() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap()
}

fn score(value: u8) -> MoodScore {
    MoodScore::new(value).unwrap()
}

async fn insert(
    repository: &SeaOrmMoodEntryRepository,
    user: i64,
    value: u8,
    at: DateTime<Utc>,
) -> i64 {
    repository
        .insert(UserId::new(user), score(value), None, at)
        .await
        .unwrap()
        .id()
}

#[tokio::test]
async fn insert_round_trips_every_field() {
    let repository = SeaOrmMoodEntryRepository::new(open().await);

    let entry = repository
        .insert(
            UserId::new(ALICE),
            score(4),
            Some("calm".to_owned()),
            noon(),
        )
        .await
        .unwrap();

    assert_eq!(entry.user_id(), UserId::new(ALICE));
    assert_eq!(entry.score().value(), 4);
    assert_eq!(entry.note(), Some("calm"));
    assert_eq!(entry.recorded_at(), noon());
    assert_eq!(entry.updated_at(), noon());
    let listed = repository
        .list_all_for_user(UserId::new(ALICE))
        .await
        .unwrap();
    assert_eq!(listed, [entry]);
}

#[tokio::test]
async fn insert_assigns_distinct_ids() {
    let repository = SeaOrmMoodEntryRepository::new(open().await);

    let first = insert(&repository, ALICE, 3, noon()).await;
    let second = insert(&repository, ALICE, 3, noon()).await;

    assert_ne!(first, second);
}

#[tokio::test]
async fn update_changes_score_note_and_updated_at_but_not_recorded_at() {
    let repository = SeaOrmMoodEntryRepository::new(open().await);
    let id = insert(&repository, ALICE, 2, noon()).await;
    let later = noon() + Duration::hours(3);

    let updated = repository
        .update(
            UserId::new(ALICE),
            id,
            score(5),
            Some("better".to_owned()),
            later,
        )
        .await
        .unwrap()
        .expect("entry exists");

    assert_eq!(updated.score().value(), 5);
    assert_eq!(updated.note(), Some("better"));
    assert_eq!(updated.recorded_at(), noon());
    assert_eq!(updated.updated_at(), later);
}

#[tokio::test]
async fn update_can_clear_the_note() {
    let repository = SeaOrmMoodEntryRepository::new(open().await);
    let id = repository
        .insert(UserId::new(ALICE), score(2), Some("x".to_owned()), noon())
        .await
        .unwrap()
        .id();

    let updated = repository
        .update(UserId::new(ALICE), id, score(2), None, noon())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.note(), None);
}

#[tokio::test]
async fn update_of_another_users_or_unknown_entry_returns_none_and_changes_nothing() {
    let repository = SeaOrmMoodEntryRepository::new(open().await);
    let id = insert(&repository, ALICE, 2, noon()).await;

    let foreign = repository
        .update(UserId::new(BOB), id, score(5), None, noon())
        .await
        .unwrap();
    let unknown = repository
        .update(UserId::new(ALICE), id + 100, score(5), None, noon())
        .await
        .unwrap();

    assert_eq!(foreign, None);
    assert_eq!(unknown, None);
    let stored = repository
        .list_all_for_user(UserId::new(ALICE))
        .await
        .unwrap();
    assert_eq!(stored[0].score().value(), 2);
}

#[tokio::test]
async fn delete_removes_only_the_owners_entry() {
    let repository = SeaOrmMoodEntryRepository::new(open().await);
    let id = insert(&repository, ALICE, 3, noon()).await;

    assert!(!repository.delete(UserId::new(BOB), id).await.unwrap());
    assert!(!repository
        .delete(UserId::new(ALICE), id + 100)
        .await
        .unwrap());
    assert_eq!(
        repository
            .list_all_for_user(UserId::new(ALICE))
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(repository.delete(UserId::new(ALICE), id).await.unwrap());
    assert!(repository
        .list_all_for_user(UserId::new(ALICE))
        .await
        .unwrap()
        .is_empty());
    assert!(!repository.delete(UserId::new(ALICE), id).await.unwrap());
}

#[tokio::test]
async fn list_is_empty_for_a_user_without_entries() {
    let repository = SeaOrmMoodEntryRepository::new(open().await);
    insert(&repository, ALICE, 3, noon()).await;

    assert!(repository
        .list_all_for_user(UserId::new(BOB))
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn list_orders_newest_first_and_breaks_ties_by_higher_id() {
    let repository = SeaOrmMoodEntryRepository::new(open().await);
    let oldest = insert(&repository, ALICE, 1, noon() - Duration::days(1)).await;
    let tie_first = insert(&repository, ALICE, 2, noon()).await;
    let tie_second = insert(&repository, ALICE, 3, noon()).await;
    insert(&repository, BOB, 4, noon() + Duration::days(1)).await;

    let ids: Vec<_> = repository
        .list_all_for_user(UserId::new(ALICE))
        .await
        .unwrap()
        .iter()
        .map(|entry| entry.id())
        .collect();

    assert_eq!(ids, [tie_second, tie_first, oldest]);
}

#[tokio::test]
async fn delete_all_for_user_reports_the_count_and_spares_other_users() {
    let repository = SeaOrmMoodEntryRepository::new(open().await);
    insert(&repository, ALICE, 1, noon()).await;
    insert(&repository, ALICE, 2, noon()).await;
    insert(&repository, BOB, 3, noon()).await;

    assert_eq!(
        repository
            .delete_all_for_user(UserId::new(ALICE))
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        repository
            .delete_all_for_user(UserId::new(ALICE))
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        repository
            .list_all_for_user(UserId::new(BOB))
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn corrupt_rows_surface_as_errors_instead_of_being_skipped() {
    let db = open().await;
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "INSERT INTO mood_tracker_entries (user_id, score, note, recorded_at, updated_at) \
         VALUES (1, 3, NULL, 'not-a-date', '2026-09-29T12:00:00+00:00')",
    ))
    .await
    .unwrap();
    let repository = SeaOrmMoodEntryRepository::new(db);

    let error = repository
        .list_all_for_user(UserId::new(ALICE))
        .await
        .unwrap_err();

    assert!(error.contains("corrupt recorded_at"), "{error}");
}

#[tokio::test]
async fn the_schema_refuses_scores_outside_one_to_five() {
    let db = open().await;

    let result = db
        .execute(Statement::from_string(
            db.get_database_backend(),
            "INSERT INTO mood_tracker_entries (user_id, score, note, recorded_at, updated_at) \
             VALUES (1, 9, NULL, '2026-09-29T12:00:00+00:00', '2026-09-29T12:00:00+00:00')",
        ))
        .await;

    assert!(result.is_err());
}
