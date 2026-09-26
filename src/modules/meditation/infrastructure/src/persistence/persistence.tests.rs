//! Repository round-trip tests against a temp SQLite database
//! (identity-infrastructure test style).
use std::{str::FromStr, time::Duration};

use chrono::{Duration as ChronoDuration, Utc};
use kernel::UserId;
use meditation_application::ports::{
    CategoryRepository, MeditationRepository, MembershipRepository, NoteRepository,
    PlaybackStateRepository, ProgramRepository, SessionRepository,
};
use sea_orm::{DatabaseConnection, SqlxSqliteConnector};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};

use crate::migrations::migrate;
use crate::persistence::{
    SeaOrmCategoryRepository, SeaOrmMeditationRepository, SeaOrmMembershipRepository,
    SeaOrmNoteRepository, SeaOrmPlaybackRepository, SeaOrmProgramRepository,
    SeaOrmSessionRepository,
};

struct TestDatabase {
    _directory: tempfile::TempDir,
    connection: DatabaseConnection,
}

impl TestDatabase {
    async fn open() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("meditation.sqlite");
        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
            .unwrap()
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(false)
            .busy_timeout(Duration::from_secs(5));
        let pool: SqlitePool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .unwrap();
        let connection = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
        migrate(&connection).await.unwrap();
        Self {
            _directory: directory,
            connection,
        }
    }
}

fn user_a() -> UserId {
    UserId::new(1)
}

fn user_b() -> UserId {
    UserId::new(2)
}

#[tokio::test]
async fn meditation_rows_round_trip_and_list_newest_first() {
    let database = TestDatabase::open().await;
    let repository = SeaOrmMeditationRepository::new(database.connection);

    let first = repository
        .insert(
            "First".to_owned(),
            "audio/ogg",
            10,
            "key-one".to_owned(),
            Utc::now(),
        )
        .await
        .unwrap();
    let second = repository
        .insert(
            "Second".to_owned(),
            "audio/ogg",
            20,
            "key-two".to_owned(),
            Utc::now() + ChronoDuration::seconds(5),
        )
        .await
        .unwrap();

    let listed = repository.list_all().await.unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].name(), "Second");
    assert_eq!(listed[1].name(), "First");

    let found = repository.find_by_id(first.id()).await.unwrap().unwrap();
    assert_eq!(found.name(), "First");
    assert!(repository.find_by_id(999).await.unwrap().is_none());
    assert!(repository.delete(second.id()).await.unwrap());
    assert!(!repository.delete(second.id()).await.unwrap());
}

#[tokio::test]
async fn category_names_are_unique_at_the_database_level() {
    let database = TestDatabase::open().await;
    let repository = SeaOrmCategoryRepository::new(database.connection);

    let created = repository
        .insert("Morning".to_owned(), Utc::now())
        .await
        .unwrap();
    assert!(repository.find_by_name("Morning").await.unwrap().is_some());

    // The UNIQUE constraint is the second gate behind the command-level
    // find_by_name check: a race that inserts twice must surface an error.
    let raced = repository.insert("Morning".to_owned(), Utc::now()).await;
    assert!(raced.is_err());
    assert!(repository.delete(created.id).await.unwrap());
}

#[tokio::test]
async fn memberships_are_idempotent_on_the_pair() {
    let database = TestDatabase::open().await;
    let meditations = SeaOrmMeditationRepository::new(database.connection.clone());
    let programs = SeaOrmProgramRepository::new(database.connection.clone());
    let memberships = SeaOrmMembershipRepository::new(database.connection);

    let meditation = meditations
        .insert("M".to_owned(), "audio/ogg", 5, "k".to_owned(), Utc::now())
        .await
        .unwrap();
    let program = programs
        .insert("P".to_owned(), None, None, Utc::now())
        .await
        .unwrap();

    let first = memberships
        .add(program.id, meditation.id(), Utc::now())
        .await;
    let second = memberships
        .add(program.id, meditation.id(), Utc::now())
        .await;
    assert_eq!(first, second, "repeat add returns the existing row id");

    let listed = memberships.list_by_program(program.id).await.unwrap();
    assert_eq!(listed.len(), 1);

    memberships
        .remove_for_meditation(meditation.id())
        .await
        .unwrap();
    let listed = memberships.list_by_program(program.id).await.unwrap();
    assert!(listed.is_empty());
}

#[tokio::test]
async fn session_triple_is_the_idempotency_key() {
    let database = TestDatabase::open().await;
    let meditations = SeaOrmMeditationRepository::new(database.connection.clone());
    let sessions = SeaOrmSessionRepository::new(database.connection);

    let meditation = meditations
        .insert("M".to_owned(), "audio/ogg", 5, "k".to_owned(), Utc::now())
        .await
        .unwrap();
    let started = Utc::now();

    let first = sessions
        .record_if_new(
            user_a(),
            meditation.id(),
            started,
            started + ChronoDuration::seconds(60),
        )
        .await
        .unwrap();
    let retry = sessions
        .record_if_new(
            user_a(),
            meditation.id(),
            started,
            started + ChronoDuration::seconds(60),
        )
        .await
        .unwrap();
    assert_eq!(first.id, retry.id);
    assert_eq!(
        sessions.count_for(user_a(), meditation.id()).await.unwrap(),
        1
    );

    // A different user or start time is a different session.
    sessions
        .record_if_new(
            user_b(),
            meditation.id(),
            started,
            started + ChronoDuration::seconds(60),
        )
        .await
        .unwrap();
    assert_eq!(
        sessions.count_for(user_b(), meditation.id()).await.unwrap(),
        1
    );
}

#[tokio::test]
async fn playback_upserts_are_per_user() {
    let database = TestDatabase::open().await;
    let meditations = SeaOrmMeditationRepository::new(database.connection.clone());
    let playback = SeaOrmPlaybackRepository::new(database.connection);

    let meditation = meditations
        .insert("M".to_owned(), "audio/ogg", 5, "k".to_owned(), Utc::now())
        .await
        .unwrap();

    playback
        .upsert_position(user_a(), meditation.id(), 30_000)
        .await
        .unwrap();
    playback
        .upsert_position(user_a(), meditation.id(), 45_000)
        .await
        .unwrap();

    let a_state = playback
        .get(user_a(), meditation.id())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(a_state.position_ms, 45_000);
    assert!(!a_state.completed);
    assert!(playback
        .get(user_b(), meditation.id())
        .await
        .unwrap()
        .is_none());

    playback
        .set_completed(user_a(), meditation.id(), true)
        .await
        .unwrap();
    playback
        .reset_position(user_a(), meditation.id())
        .await
        .unwrap();
    let a_state = playback
        .get(user_a(), meditation.id())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(a_state.position_ms, 0);
    assert!(a_state.completed);
}

#[tokio::test]
async fn notes_stay_author_scoped_and_newest_first() {
    let database = TestDatabase::open().await;
    let meditations = SeaOrmMeditationRepository::new(database.connection.clone());
    let notes = SeaOrmNoteRepository::new(database.connection);

    let meditation = meditations
        .insert("M".to_owned(), "audio/ogg", 5, "k".to_owned(), Utc::now())
        .await
        .unwrap();
    let older = notes
        .insert(user_a(), meditation.id(), "older".to_owned(), Utc::now())
        .await
        .unwrap();
    notes
        .insert(user_a(), meditation.id(), "newer".to_owned(), Utc::now())
        .await
        .unwrap();

    let listed = notes.list_for(user_a(), meditation.id()).await.unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].text, "newer");
    assert!(notes
        .list_for(user_b(), meditation.id())
        .await
        .unwrap()
        .is_empty());

    // B cannot touch A's note.
    assert!(!notes
        .update(older.id, user_b(), "hijack".to_owned(), Utc::now())
        .await
        .unwrap());
    assert!(!notes.delete(older.id, user_b()).await.unwrap());
    assert!(notes
        .update(older.id, user_a(), "edited".to_owned(), Utc::now())
        .await
        .unwrap());

    // Admin meditation delete sweeps every author's notes.
    assert_eq!(
        notes.delete_for_meditation(meditation.id()).await.unwrap(),
        2
    );
}

#[tokio::test]
async fn program_category_sweep_nulls_references() {
    let database = TestDatabase::open().await;
    let programs = SeaOrmProgramRepository::new(database.connection.clone());
    let categories = SeaOrmCategoryRepository::new(database.connection);

    let category = categories
        .insert("Evening".to_owned(), Utc::now())
        .await
        .unwrap();
    let program = programs
        .insert("P".to_owned(), None, None, Utc::now())
        .await
        .unwrap();
    programs
        .set_category(program.id, Some(category.id))
        .await
        .unwrap();

    assert_eq!(
        programs.clear_category(category.id).await.unwrap(),
        1,
        "sweep nulls the program's reference"
    );
    let stored = programs.find_by_id(program.id).await.unwrap().unwrap();
    assert_eq!(stored.category_id, None);
}
