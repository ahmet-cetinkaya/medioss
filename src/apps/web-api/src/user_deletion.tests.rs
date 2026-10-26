//! Cross-module user deletion over real SQLite repositories and a real
//! media root: every module's rows vanish for the deleted user, every other
//! user's rows survive, and the purge is idempotent.

use std::sync::Arc;

use chrono::{Duration, Utc};
use identity_presentation::UserDeletionListener;
use kernel::UserId;
use meditation_application::ports::{
    FavoriteRepository, NoteRepository, PlaybackStateRepository, SessionRepository,
};
use meditation_infrastructure::persistence::{
    SeaOrmFavoriteRepository, SeaOrmNoteRepository, SeaOrmPlaybackRepository,
    SeaOrmSessionRepository,
};
use mood_tracker_application::ports::MoodEntryRepository;
use mood_tracker_domain::MoodScore;
use mood_tracker_infrastructure::persistence::SeaOrmMoodEntryRepository;
use sea_orm::{DatabaseConnection, SqlxSqliteConnector};
use soundscape_application::ports::{MixRepository, NewMix};
use soundscape_domain::MixItem;
use soundscape_infrastructure::persistence::SeaOrmMixRepository;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use super::CrossModuleUserDeletion;

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
        soundscape_infrastructure::migrations::migrate(&connection)
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

/// A connection whose schema was never migrated: any repository query
/// against it fails, which stands in for a broken store.
async fn unmigrated_connection() -> DatabaseConnection {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    SqlxSqliteConnector::from_sqlx_sqlite_pool(pool)
}

fn notes(database: &TestDatabase) -> SeaOrmNoteRepository {
    SeaOrmNoteRepository::new(database.connection.clone())
}

fn favorites(database: &TestDatabase) -> SeaOrmFavoriteRepository {
    SeaOrmFavoriteRepository::new(database.connection.clone())
}

fn playback(database: &TestDatabase) -> SeaOrmPlaybackRepository {
    SeaOrmPlaybackRepository::new(database.connection.clone())
}

fn sessions(database: &TestDatabase) -> SeaOrmSessionRepository {
    SeaOrmSessionRepository::new(database.connection.clone())
}

fn mixes(database: &TestDatabase) -> SeaOrmMixRepository {
    SeaOrmMixRepository::new(database.connection.clone())
}

fn moods(database: &TestDatabase) -> SeaOrmMoodEntryRepository {
    SeaOrmMoodEntryRepository::new(database.connection.clone())
}

fn listener(
    database: &TestDatabase,
    notes_store: Arc<dyn NoteRepository>,
    media_root: &std::path::Path,
) -> CrossModuleUserDeletion {
    CrossModuleUserDeletion::new(
        notes_store,
        Arc::new(favorites(database)),
        Arc::new(playback(database)),
        Arc::new(sessions(database)),
        Arc::new(mixes(database)),
        Arc::new(moods(database)),
        media_root.to_path_buf(),
    )
}

async fn seed_note(database: &TestDatabase, user_id: UserId, text: &str) {
    notes(database)
        .insert(user_id, 1, text.to_owned(), Utc::now())
        .await
        .unwrap();
}

async fn seed_favorite(database: &TestDatabase, user_id: UserId) {
    favorites(database)
        .add(user_id, 1, Utc::now())
        .await
        .unwrap();
}

async fn seed_session(database: &TestDatabase, user_id: UserId) {
    let started = Utc::now();
    sessions(database)
        .record_if_new(user_id, 1, started, started + Duration::minutes(10))
        .await
        .unwrap();
}

async fn seed_playback(database: &TestDatabase, user_id: UserId, position_ms: i64) {
    playback(database)
        .upsert_position(user_id, 1, position_ms)
        .await
        .unwrap();
}

async fn seed_mix(database: &TestDatabase, user_id: UserId, name: &str) {
    mixes(database)
        .create(NewMix {
            user_id,
            name: name.to_owned(),
            items: vec![MixItem {
                sound_id: 1,
                volume: 50,
                position: 0,
            }],
            now: Utc::now(),
        })
        .await
        .unwrap();
}

async fn seed_mood(database: &TestDatabase, user_id: UserId, note: &str) {
    moods(database)
        .insert(
            user_id,
            MoodScore::new(4).unwrap(),
            Some(note.to_owned()),
            Utc::now(),
        )
        .await
        .unwrap();
}

async fn seed_media(media_root: &std::path::Path, user_id: UserId) -> std::path::PathBuf {
    let blob = media_root
        .join(user_id.value().to_string())
        .join("cover.webp");
    tokio::fs::create_dir_all(blob.parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(&blob, b"cover-bytes").await.unwrap();
    blob
}

fn media_dir(media_root: &std::path::Path, user_id: UserId) -> std::path::PathBuf {
    media_root.join(user_id.value().to_string())
}

#[tokio::test]
async fn on_user_deleted_purges_every_module_but_leaves_other_users_intact() {
    let database = TestDatabase::open().await;
    let media_root = database._directory.path().join("media");
    let gone = UserId::new(7);
    let kept = UserId::new(8);

    seed_note(&database, gone, "gone note").await;
    seed_note(&database, kept, "kept note").await;
    seed_session(&database, gone).await;
    seed_session(&database, kept).await;
    seed_favorite(&database, gone).await;
    seed_favorite(&database, kept).await;
    seed_playback(&database, gone, 3_000).await;
    seed_playback(&database, kept, 4_000).await;
    seed_mix(&database, gone, "Gone Mix").await;
    seed_mix(&database, kept, "Kept Mix").await;
    seed_mood(&database, gone, "gone mood").await;
    seed_mood(&database, kept, "kept mood").await;
    let gone_blob = seed_media(&media_root, gone).await;
    let kept_blob = seed_media(&media_root, kept).await;

    let deletion = listener(&database, Arc::new(notes(&database)), &media_root);
    deletion.on_user_deleted(gone).await.unwrap();

    assert!(notes(&database)
        .list_for_user(gone)
        .await
        .unwrap()
        .is_empty());
    assert_eq!(notes(&database).list_for_user(kept).await.unwrap().len(), 1);
    assert!(sessions(&database)
        .list_for_user(gone)
        .await
        .unwrap()
        .is_empty());
    assert_eq!(
        sessions(&database).list_for_user(kept).await.unwrap().len(),
        1
    );
    assert!(favorites(&database)
        .list_for_user(gone)
        .await
        .unwrap()
        .is_empty());
    assert_eq!(
        favorites(&database).list_for_user(kept).await.unwrap(),
        vec![1]
    );
    assert_eq!(playback(&database).get(gone, 1).await.unwrap(), None);
    assert_eq!(
        playback(&database).get(kept, 1).await.unwrap(),
        Some(meditation_application::ports::PlaybackState {
            position_ms: 4_000,
            completed: false,
        })
    );
    assert!(mixes(&database).list_for(gone).await.unwrap().is_empty());
    assert_eq!(mixes(&database).list_for(kept).await.unwrap().len(), 1);
    assert!(moods(&database)
        .list_all_for_user(gone)
        .await
        .unwrap()
        .is_empty());
    assert_eq!(
        moods(&database)
            .list_all_for_user(kept)
            .await
            .unwrap()
            .len(),
        1
    );

    assert!(!media_dir(&media_root, gone).exists());
    assert!(!gone_blob.exists());
    assert!(kept_blob.exists());
}

#[tokio::test]
async fn on_user_deleted_is_idempotent() {
    let database = TestDatabase::open().await;
    let media_root = database._directory.path().join("media");
    let gone = UserId::new(7);
    let kept = UserId::new(8);

    seed_note(&database, gone, "gone note").await;
    seed_note(&database, kept, "kept note").await;
    seed_mix(&database, gone, "Gone Mix").await;
    seed_mix(&database, kept, "Kept Mix").await;
    seed_media(&media_root, gone).await;

    let deletion = listener(&database, Arc::new(notes(&database)), &media_root);
    deletion.on_user_deleted(gone).await.unwrap();
    // A retried purge over already-purged state (and an absent media
    // folder) must still succeed and must not touch other users.
    deletion.on_user_deleted(gone).await.unwrap();

    assert!(notes(&database)
        .list_for_user(gone)
        .await
        .unwrap()
        .is_empty());
    assert_eq!(notes(&database).list_for_user(kept).await.unwrap().len(), 1);
    assert!(mixes(&database).list_for(gone).await.unwrap().is_empty());
    assert_eq!(mixes(&database).list_for(kept).await.unwrap().len(), 1);
    assert_eq!(
        notes(&database).list_for_user(kept).await.unwrap()[0].text,
        "kept note"
    );
}

#[tokio::test]
async fn on_user_deleted_reports_the_failing_store_and_keeps_media() {
    let database = TestDatabase::open().await;
    let media_root = database._directory.path().join("media");
    let gone = UserId::new(7);
    let gone_blob = seed_media(&media_root, gone).await;

    let deletion = listener(
        &database,
        Arc::new(SeaOrmNoteRepository::new(unmigrated_connection().await)),
        &media_root,
    );
    let error = deletion.on_user_deleted(gone).await.unwrap_err();

    assert!(
        error.starts_with("meditation notes purge failed"),
        "unexpected error: {error}"
    );
    // The ordered purge aborts on the first failure, so the media wipe
    // after it never runs.
    assert!(gone_blob.exists());
}
