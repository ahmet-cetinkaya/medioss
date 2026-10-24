//! Composition-root portability adapters over real SQLite repositories:
//! mood records round-trip, practice snapshots join the meditation name
//! and duration onto each row while skipping orphans, and the display-name
//! resolver reports a vanished user instead of returning `Ok(None)`.

use chrono::{Duration, Utc};
use identity_application::ports::{MemberRegistration, UserRepository};
use identity_domain::{Email, PasswordHash};
use kernel::UserId;
use meditation_application::ports::MeditationRepository;
use meditation_infrastructure::persistence::{
    SeaOrmMeditationRepository, SeaOrmNoteRepository, SeaOrmPlaybackRepository,
    SeaOrmSessionRepository,
};
use sea_orm::{ConnectionTrait, DatabaseConnection, SqlxSqliteConnector};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use super::{Library, MoodRecords, PracticeRecords, ProfileNames};
use mood_tracker_infrastructure::persistence::SeaOrmMoodEntryRepository;
use portability_application::models::mood_record::MoodRecord;
use portability_application::ports::display_name_resolver::DisplayNameResolver;
use portability_application::ports::meditation_lookup::MeditationLookup;
use portability_application::ports::mood_record_sink::MoodRecordSink as _;
use portability_application::ports::own_mood_records::OwnMoodRecords as _;
use portability_application::ports::own_practice_records::OwnPracticeRecords as _;
use portability_application::ports::practice_sink::PracticeSink as _;

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
        identity_infrastructure::migrate(&connection).await.unwrap();
        meditation_infrastructure::migrations::migrate(&connection)
            .await
            .unwrap();
        mood_tracker_infrastructure::migrations::migrate(&connection)
            .await
            .unwrap();
        Self {
            _directory: directory,
            connection,
        }
    }
}

fn mood_records(database: &TestDatabase) -> MoodRecords {
    MoodRecords {
        entries: SeaOrmMoodEntryRepository::new(database.connection.clone()),
    }
}

fn practice_records(database: &TestDatabase) -> PracticeRecords {
    PracticeRecords {
        meditations: SeaOrmMeditationRepository::new(database.connection.clone()),
        sessions: SeaOrmSessionRepository::new(database.connection.clone()),
        notes: SeaOrmNoteRepository::new(database.connection.clone()),
        playback: SeaOrmPlaybackRepository::new(database.connection.clone()),
    }
}

async fn seed_meditation(database: &TestDatabase, name: &str, key: &str, duration_ms: i64) -> i64 {
    SeaOrmMeditationRepository::new(database.connection.clone())
        .insert(
            name.to_owned(),
            "audio/ogg",
            10,
            key.to_owned(),
            duration_ms,
            Utc::now(),
            None,
        )
        .await
        .unwrap()
        .id()
}

async fn seed_user(database: &TestDatabase, user_id: UserId, email: &str, name: Option<&str>) {
    let users = identity_infrastructure::SeaOrmUserRepository::new(database.connection.clone());
    users
        .insert_member(MemberRegistration::new(
            Email::parse(email).unwrap(),
            PasswordHash::from_stored("test-hash".to_owned()),
            Utc::now(),
        ))
        .await
        .unwrap();
    // AUTOINCREMENT assigned the real row; this test pins the id the
    // adapter receives by rewriting it (no cross-module FK allows it).
    database
        .connection
        .execute_unprepared(&format!(
            "UPDATE users SET id = {} WHERE email = '{}'",
            user_id.value(),
            email
        ))
        .await
        .unwrap();
    if let Some(name) = name {
        users
            .set_profile(
                user_id,
                Some(identity_domain::DisplayName::parse(name).unwrap()),
                None,
            )
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn mood_records_round_trip_through_the_sink_and_source() {
    let database = TestDatabase::open().await;
    let user = UserId::new(4);
    let recorded_at = Utc::now() - Duration::days(1);
    let records = mood_records(&database);

    records
        .insert(
            user,
            MoodRecord {
                score: 3,
                note: Some("steady".to_owned()),
                recorded_at,
            },
        )
        .await
        .unwrap();

    let listed = records.list_for_user(user).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].score, 3);
    assert_eq!(listed[0].note.as_deref(), Some("steady"));
    assert_eq!(listed[0].recorded_at, recorded_at);

    // A score outside the domain range is rejected by the sink.
    let error = records
        .insert(
            user,
            MoodRecord {
                score: 9,
                note: None,
                recorded_at,
            },
        )
        .await
        .unwrap_err();
    assert!(error.contains("imported score invalid"), "{error}");
}

#[tokio::test]
async fn practice_snapshot_joins_names_and_durations_and_skips_orphans() {
    let database = TestDatabase::open().await;
    let user = UserId::new(4);
    let orphaned = seed_meditation(&database, "Temporary", "media/temporary.opus", 1_000).await;
    let kept = seed_meditation(&database, "Evening Calm", "media/evening-calm.opus", 7_500).await;
    let practice = practice_records(&database);

    let started_at = Utc::now() - Duration::hours(1);
    let finished_at = started_at + Duration::minutes(12);
    practice
        .record_session(user, orphaned, started_at, finished_at)
        .await
        .unwrap();
    practice
        .record_session(user, kept, started_at, finished_at)
        .await
        .unwrap();
    practice
        .insert_note(user, orphaned, "orphan", Utc::now())
        .await
        .unwrap();
    practice
        .insert_note(user, kept, "kept note", Utc::now())
        .await
        .unwrap();
    practice.upsert_position(user, kept, 2_500).await.unwrap();
    practice.set_completed(user, kept, true).await.unwrap();
    SeaOrmMeditationRepository::new(database.connection.clone())
        .delete(orphaned)
        .await
        .unwrap();

    let snapshot = practice.list_for_user(user).await.unwrap();

    assert_eq!(snapshot.sessions.len(), 1, "orphan session must be skipped");
    assert_eq!(snapshot.sessions[0].meditation_name, "Evening Calm");
    assert_eq!(snapshot.sessions[0].meditation_duration_ms, 7_500);
    assert_eq!(snapshot.sessions[0].started_at, started_at);
    assert_eq!(snapshot.sessions[0].finished_at, finished_at);
    assert_eq!(snapshot.notes.len(), 1, "orphan note must be skipped");
    assert_eq!(snapshot.notes[0].meditation_name, "Evening Calm");
    assert_eq!(snapshot.notes[0].text, "kept note");
    assert_eq!(snapshot.playback.len(), 1);
    assert_eq!(snapshot.playback[0].meditation_name, "Evening Calm");
    assert_eq!(snapshot.playback[0].position_ms, 2_500);
    assert!(snapshot.playback[0].completed);
}

#[tokio::test]
async fn practice_sink_upserts_playback_state_in_place() {
    let database = TestDatabase::open().await;
    let user = UserId::new(4);
    let meditation = seed_meditation(&database, "Morning", "media/morning.opus", 3_000).await;
    let practice = practice_records(&database);

    practice
        .upsert_position(user, meditation, 1_000)
        .await
        .unwrap();
    practice
        .upsert_position(user, meditation, 2_000)
        .await
        .unwrap();
    practice
        .set_completed(user, meditation, true)
        .await
        .unwrap();
    practice
        .set_completed(user, meditation, true)
        .await
        .unwrap();

    let snapshot = practice.list_for_user(user).await.unwrap();
    assert_eq!(
        snapshot.playback.len(),
        1,
        "upserts must not duplicate rows"
    );
    assert_eq!(snapshot.playback[0].position_ms, 2_000);
    assert!(snapshot.playback[0].completed);
}

#[tokio::test]
async fn display_name_resolves_the_profile_and_reports_a_vanished_user() {
    let database = TestDatabase::open().await;
    let user = UserId::new(4);
    seed_user(&database, user, "ada@example.com", Some("Ada")).await;
    let resolver = ProfileNames {
        users: identity_infrastructure::SeaOrmUserRepository::new(database.connection.clone()),
    };

    assert_eq!(
        resolver.display_name(user).await.unwrap(),
        Some("Ada".to_owned())
    );

    let error = resolver.display_name(UserId::new(99)).await.unwrap_err();
    assert_eq!(error, "user vanished");
}

#[tokio::test]
async fn library_maps_meditations_to_refs() {
    let database = TestDatabase::open().await;
    seed_meditation(&database, "Body Scan", "media/body-scan.opus", 6_000).await;
    seed_meditation(&database, "Evening Calm", "media/evening-calm.opus", 7_500).await;
    let library = Library {
        meditations: SeaOrmMeditationRepository::new(database.connection.clone()),
    };

    let mut references = library.list_all().await.unwrap();
    references.sort_by_key(|reference| reference.id);

    assert_eq!(references.len(), 2);
    assert_eq!(references[0].name, "Body Scan");
    assert_eq!(references[0].duration_ms, 6_000);
    assert_eq!(references[1].name, "Evening Calm");
    assert_eq!(references[1].duration_ms, 7_500);
}
