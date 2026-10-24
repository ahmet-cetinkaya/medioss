//! Composition-root mood-tracker wiring: the wall clock reads between the
//! caller's `Utc::now` bounds, and the timeline adapter maps real
//! note/session rows into timeline events while skipping orphans whose
//! meditation row is gone.

use chrono::{Duration, Utc};
use kernel::UserId;
use meditation_application::ports::{MeditationRepository, NoteRepository, SessionRepository};
use meditation_infrastructure::persistence::{
    SeaOrmMeditationRepository, SeaOrmNoteRepository, SeaOrmSessionRepository,
};
use mood_tracker_application::ports::{
    Clock, MeditationTimelineEventKind, MeditationTimelineSource,
};
use sea_orm::{DatabaseConnection, SqlxSqliteConnector};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use super::{MeditationTimelineAdapter, SystemClock};

struct TestDatabase {
    _directory: tempfile::TempDir,
    connection: DatabaseConnection,
}

impl TestDatabase {
    async fn open() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let pool = SqlitePoolOptions::new()
            .max_connections(2)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(directory.path().join("test.sqlite"))
                    .create_if_missing(true),
            )
            .await
            .unwrap();
        let connection = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
        meditation_infrastructure::migrations::migrate(&connection)
            .await
            .unwrap();
        Self {
            _directory: directory,
            connection,
        }
    }
}

fn adapter(database: &TestDatabase) -> MeditationTimelineAdapter {
    MeditationTimelineAdapter {
        meditations: SeaOrmMeditationRepository::new(database.connection.clone()),
        notes: SeaOrmNoteRepository::new(database.connection.clone()),
        sessions: SeaOrmSessionRepository::new(database.connection.clone()),
    }
}

async fn seed_meditation(database: &TestDatabase, name: &str, key: &str) -> i64 {
    SeaOrmMeditationRepository::new(database.connection.clone())
        .insert(
            name.to_owned(),
            "audio/ogg",
            10,
            key.to_owned(),
            6_000,
            Utc::now(),
            None,
        )
        .await
        .unwrap()
        .id()
}

#[tokio::test]
async fn system_clock_reads_between_the_callers_bounds() {
    let clock = SystemClock;
    let before = Utc::now();
    let now = clock.now().await;
    let after = Utc::now();

    assert!(before <= now && now <= after);
}

#[tokio::test]
async fn timeline_maps_notes_and_sessions_with_their_meditation_names() {
    let database = TestDatabase::open().await;
    let meditation = seed_meditation(&database, "Body Scan", "media/body-scan.opus").await;
    let user = UserId::new(3);
    let source = adapter(&database);

    let note_recorded_at = Utc::now() - Duration::hours(2);
    let note = source
        .notes
        .insert(
            user,
            meditation,
            "Shoulders relaxed".to_owned(),
            note_recorded_at,
        )
        .await
        .unwrap();
    let started_at = Utc::now() - Duration::hours(1);
    let finished_at = started_at + Duration::minutes(10);
    source
        .sessions
        .record_if_new(user, meditation, started_at, finished_at)
        .await
        .unwrap();

    let events = source.list_for_user(user).await.unwrap();

    assert_eq!(events.len(), 2);
    let note_event = events
        .iter()
        .find(|event| event.kind == MeditationTimelineEventKind::Note)
        .unwrap();
    assert_eq!(note_event.id, note.id);
    assert_eq!(note_event.meditation_name, "Body Scan");
    assert_eq!(note_event.note.as_deref(), Some("Shoulders relaxed"));
    assert_eq!(note_event.recorded_at, note_recorded_at);
    let session_event = events
        .iter()
        .find(|event| event.kind == MeditationTimelineEventKind::CompletedSession)
        .unwrap();
    assert_eq!(session_event.meditation_name, "Body Scan");
    assert_eq!(session_event.note, None);
    assert_eq!(session_event.recorded_at, finished_at);
}

#[tokio::test]
async fn timeline_skips_notes_and_sessions_whose_meditation_is_gone() {
    let database = TestDatabase::open().await;
    let orphaned = seed_meditation(&database, "Temporary", "media/temporary.opus").await;
    let kept = seed_meditation(&database, "Evening Calm", "media/evening-calm.opus").await;
    let user = UserId::new(3);
    let source = adapter(&database);

    let now = Utc::now();
    source
        .notes
        .insert(user, orphaned, "orphan note".to_owned(), now)
        .await
        .unwrap();
    source
        .notes
        .insert(user, kept, "kept note".to_owned(), now)
        .await
        .unwrap();
    source
        .sessions
        .record_if_new(user, orphaned, now, now)
        .await
        .unwrap();
    source
        .sessions
        .record_if_new(user, kept, now, now + Duration::minutes(5))
        .await
        .unwrap();

    // No cross-module foreign keys: the owning module sweeps notes on
    // delete, but a session row (or a lagging note) can outlive its
    // meditation. Drop the row directly to reproduce the orphan.
    SeaOrmMeditationRepository::new(database.connection.clone())
        .delete(orphaned)
        .await
        .unwrap();

    let events = source.list_for_user(user).await.unwrap();

    assert_eq!(events.len(), 2, "orphan rows must be skipped: {events:?}");
    assert!(events
        .iter()
        .all(|event| event.meditation_name == "Evening Calm"));
}

#[tokio::test]
async fn timeline_is_user_scoped() {
    let database = TestDatabase::open().await;
    let meditation = seed_meditation(&database, "Morning", "media/morning.opus").await;
    let source = adapter(&database);

    source
        .notes
        .insert(UserId::new(1), meditation, "mine".to_owned(), Utc::now())
        .await
        .unwrap();

    assert!(source
        .list_for_user(UserId::new(2))
        .await
        .unwrap()
        .is_empty());
    assert_eq!(source.list_for_user(UserId::new(1)).await.unwrap().len(), 1);
}
