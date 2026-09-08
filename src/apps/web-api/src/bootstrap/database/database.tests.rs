use std::{fs, io::Write, os::unix::fs::PermissionsExt, time::Duration};

use sqlx::{pool::PoolConnection, Sqlite};

use sea_orm::DbErr;

use super::open;
use crate::bootstrap::BootstrapError;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
const QA_PREFIX: &str = "manual QA";

#[derive(Debug, PartialEq, Eq)]
struct Pragmas {
    journal_mode: String,
    foreign_keys: i64,
    busy_timeout: i64,
}

async fn read_pragmas(connection: &mut PoolConnection<Sqlite>) -> Pragmas {
    Pragmas {
        journal_mode: sqlx::query_scalar("PRAGMA journal_mode")
            .fetch_one(&mut **connection)
            .await
            .unwrap(),
        foreign_keys: sqlx::query_scalar("PRAGMA foreign_keys")
            .fetch_one(&mut **connection)
            .await
            .unwrap(),
        busy_timeout: sqlx::query_scalar("PRAGMA busy_timeout")
            .fetch_one(&mut **connection)
            .await
            .unwrap(),
    }
}

#[tokio::test]
async fn every_pool_connection_uses_required_pragmas() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("medioss.db");
    let database = open(&path).await.unwrap();
    let mut first_connection = database.pool.acquire().await.unwrap();
    let mut second_connection = database.pool.acquire().await.unwrap();

    let first_pragmas = read_pragmas(&mut first_connection).await;
    let second_pragmas = read_pragmas(&mut second_connection).await;

    writeln!(
        std::io::stdout(),
        "{QA_PREFIX} connection 1: {first_pragmas:?}"
    )
    .unwrap();
    writeln!(
        std::io::stdout(),
        "{QA_PREFIX} connection 2: {second_pragmas:?}"
    )
    .unwrap();
    assert_eq!(
        first_pragmas,
        Pragmas {
            journal_mode: "wal".to_owned(),
            foreign_keys: 1,
            busy_timeout: 5_000,
        }
    );
    assert_eq!(first_pragmas, second_pragmas);

    drop(first_connection);
    drop(second_connection);
    database.pool.close().await;
    assert!(database.connection.get_sqlite_connection_pool().is_closed());
}

#[tokio::test]
async fn opening_database_creates_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("medioss.db");

    assert!(!path.exists());
    let database = open(&path).await.unwrap();
    assert!(path.is_file());

    database.pool.close().await;
}

#[tokio::test]
async fn unwritable_database_path_returns_typed_error_with_exact_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let unwritable_dir = temp_dir.path().join("unwritable");
    fs::create_dir(&unwritable_dir).unwrap();
    fs::set_permissions(&unwritable_dir, fs::Permissions::from_mode(0o555)).unwrap();
    let path = unwritable_dir.join("medioss.db");

    let result = open(&path).await;
    fs::set_permissions(&unwritable_dir, fs::Permissions::from_mode(0o755)).unwrap();

    match result {
        Err(BootstrapError::DatabaseOpen {
            path: error_path,
            source,
        }) => {
            assert_eq!(error_path, path);
            assert!(!source.to_string().is_empty());
        }
        Err(error) => panic!("expected DatabaseOpen, got {error:?}"),
        Ok(_) => panic!("expected invalid path to fail"),
    }
}

#[tokio::test]
async fn concurrent_database_starts_complete_within_bound() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("medioss.db");

    let (first, second) = tokio::time::timeout(STARTUP_TIMEOUT, async {
        tokio::join!(open(&path), open(&path))
    })
    .await
    .expect("concurrent database startup timed out");

    let first = first.unwrap();
    let second = second.unwrap();
    first.pool.close().await;
    second.pool.close().await;
}

#[test]
fn migration_error_preserves_path_and_source() {
    let path = std::path::PathBuf::from("/data/medioss.db");
    let error = BootstrapError::Migration {
        path: path.clone(),
        source: DbErr::Custom("migration failed".to_owned()),
    };

    match error {
        BootstrapError::Migration {
            path: error_path,
            source: DbErr::Custom(message),
        } => {
            assert_eq!(error_path, path);
            assert_eq!(message, "migration failed");
        }
        other => panic!("expected Migration, got {other:?}"),
    }
}
