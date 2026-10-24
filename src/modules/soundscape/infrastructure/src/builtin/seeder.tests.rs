use std::path::Path;
use std::str::FromStr;

use sea_orm::{ConnectionTrait, DatabaseConnection, SqlxSqliteConnector, Statement};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use super::{display_name, seed, seed_if_present, slug_for};
use crate::migrations::migrate;

struct TestDatabase {
    _directory: tempfile::TempDir,
    connection: DatabaseConnection,
}

impl TestDatabase {
    async fn open() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("soundscape.sqlite");
        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
            .unwrap()
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
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

    async fn scalar(&self, sql: &str) -> i64 {
        self.connection
            .query_one(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                sql.to_owned(),
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get_by_index::<i64>(0)
            .unwrap()
    }
}

fn write_manifest(dir: &Path, json: &str) {
    std::fs::write(dir.join("manifest.json"), json).unwrap();
}

const RAIN_MANIFEST: &str = r#"[
  {"category":" Rain ","category_name":"Rain","name":"heavy-rain","file":"heavy-rain.opus",
   "duration_ms":4000,"size":3,"content_type":"audio/ogg"},
  {"category":"rain","category_name":"Rain","name":"light-rain","file":"light-rain.opus",
   "duration_ms":2000,"size":3,"content_type":"audio/ogg"}
]"#;

fn builtin_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    write_manifest(dir.path(), RAIN_MANIFEST);
    std::fs::write(dir.path().join("heavy-rain.opus"), b"aaa").unwrap();
    std::fs::write(dir.path().join("light-rain.opus"), b"bbb").unwrap();
    dir
}

async fn run_seed(db: &TestDatabase, dir: &Path, media: &Path) -> Result<(), super::SeedError> {
    seed(&db.connection, media, dir, &dir.join("manifest.json")).await
}

#[test]
fn slug_is_trimmed_and_lowercased() {
    assert_eq!(slug_for("  Rain "), "rain");
}

#[test]
fn display_name_title_cases_dash_separated_words() {
    assert_eq!(display_name("heavy-rain"), "Heavy Rain");
    assert_eq!(display_name("--a--b-"), "A B");
    assert_eq!(display_name(""), "");
}

#[tokio::test]
async fn seed_creates_one_category_and_all_sounds_with_stored_blobs() {
    let db = TestDatabase::open().await;
    let dir = builtin_dir();
    let media = tempfile::tempdir().unwrap();

    run_seed(&db, dir.path(), media.path()).await.unwrap();

    assert_eq!(
        db.scalar("SELECT COUNT(*) FROM soundscape_categories WHERE slug='rain'")
            .await,
        1
    );
    assert_eq!(db.scalar("SELECT COUNT(*) FROM soundscape_sounds").await, 2);
    assert_eq!(
        db.scalar("SELECT COUNT(*) FROM soundscape_sounds WHERE name='Heavy Rain' AND builtin_id='rain:heavy-rain' AND icon_key='rain' AND duration_ms=4000").await,
        1
    );
    assert!(media.path().join("soundscape").is_dir());
}

#[tokio::test]
async fn seeding_twice_does_not_duplicate_sounds() {
    let db = TestDatabase::open().await;
    let dir = builtin_dir();
    let media = tempfile::tempdir().unwrap();

    run_seed(&db, dir.path(), media.path()).await.unwrap();
    run_seed(&db, dir.path(), media.path()).await.unwrap();

    assert_eq!(db.scalar("SELECT COUNT(*) FROM soundscape_sounds").await, 2);
    assert_eq!(
        db.scalar("SELECT COUNT(*) FROM soundscape_categories WHERE slug='rain'")
            .await,
        1
    );
}

#[tokio::test]
async fn tombstoned_builtin_sounds_are_not_resurrected() {
    let db = TestDatabase::open().await;
    let dir = builtin_dir();
    let media = tempfile::tempdir().unwrap();
    db.connection
        .execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "INSERT INTO soundscape_builtin_tombstones(builtin_id, deleted_at) VALUES('rain:heavy-rain','2026-01-01T00:00:00Z')".to_owned(),
        ))
        .await
        .unwrap();

    run_seed(&db, dir.path(), media.path()).await.unwrap();

    assert_eq!(db.scalar("SELECT COUNT(*) FROM soundscape_sounds").await, 1);
    assert_eq!(
        db.scalar("SELECT COUNT(*) FROM soundscape_sounds WHERE builtin_id='rain:light-rain'")
            .await,
        1
    );
}

#[tokio::test]
async fn invalid_manifest_json_is_a_manifest_error() {
    let db = TestDatabase::open().await;
    let dir = tempfile::tempdir().unwrap();
    write_manifest(dir.path(), "not json");
    let media = tempfile::tempdir().unwrap();

    let error = run_seed(&db, dir.path(), media.path()).await.unwrap_err();

    assert!(matches!(error, super::SeedError::Manifest(_)));
}

#[tokio::test]
async fn missing_audio_file_is_an_io_error() {
    let db = TestDatabase::open().await;
    let dir = tempfile::tempdir().unwrap();
    write_manifest(dir.path(), RAIN_MANIFEST);
    let media = tempfile::tempdir().unwrap();

    let error = run_seed(&db, dir.path(), media.path()).await.unwrap_err();

    assert!(matches!(error, super::SeedError::Io(_)));
}

// The only test that touches MEDIOSS_BUILTIN_SOUNDS_DIR, so it cannot race.
#[tokio::test]
async fn seed_if_present_uses_the_env_dir_and_reports_whether_it_seeded() {
    let db = TestDatabase::open().await;
    let media = tempfile::tempdir().unwrap();
    let empty = tempfile::tempdir().unwrap();
    std::env::set_var("MEDIOSS_BUILTIN_SOUNDS_DIR", empty.path());
    assert!(!seed_if_present(&db.connection, media.path()).await.unwrap());

    let dir = builtin_dir();
    std::env::set_var("MEDIOSS_BUILTIN_SOUNDS_DIR", dir.path());
    let seeded = seed_if_present(&db.connection, media.path()).await.unwrap();
    std::env::remove_var("MEDIOSS_BUILTIN_SOUNDS_DIR");

    assert!(seeded);
    assert_eq!(db.scalar("SELECT COUNT(*) FROM soundscape_sounds").await, 2);
}
