use std::str::FromStr;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use identity_application::ports::{LoginThrottleRepository, RepositoryError};
use identity_domain::{Email, PasswordHash};
use sea_orm::{DatabaseConnection, SqlxSqliteConnector};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};

use super::SeaOrmUserRepository;
use crate::migrate;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(5);

struct TestDatabase {
    _directory: tempfile::TempDir,
    connection: DatabaseConnection,
}

impl TestDatabase {
    async fn open() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("login-throttle.sqlite");
        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
            .unwrap()
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true)
            .busy_timeout(STARTUP_TIMEOUT);
        let pool: SqlitePool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .unwrap();
        let connection = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool.clone());
        migrate(&connection).await.unwrap();
        Self {
            _directory: directory,
            connection,
        }
    }

    fn repository(&self) -> SeaOrmUserRepository {
        SeaOrmUserRepository::new(self.connection.clone())
    }
}

fn at(minutes: u64) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 24, 12, 0, 0).unwrap() + chrono::TimeDelta::minutes(minutes as i64)
}

#[tokio::test]
async fn unknown_identifier_reports_no_attempts() {
    let database = TestDatabase::open().await;
    let repository = database.repository();

    assert_eq!(
        repository.failed_attempts("nobody@example.com").await.unwrap(),
        None
    );
}

#[tokio::test]
async fn failures_accumulate_and_clearing_removes_the_row() {
    let database = TestDatabase::open().await;
    let repository = database.repository();

    let first = repository
        .record_failed_attempt("throttled@example.com", at(0))
        .await
        .unwrap();
    assert_eq!(first.count(), 1);
    assert_eq!(first.last_attempt_at(), at(0));

    let second = repository
        .record_failed_attempt("throttled@example.com", at(1))
        .await
        .unwrap();
    assert_eq!(second.count(), 2);

    let reloaded = repository
        .failed_attempts("throttled@example.com")
        .await
        .unwrap()
        .expect("recorded attempts must persist");
    assert_eq!(reloaded.count(), 2);
    assert_eq!(reloaded.last_attempt_at(), at(1));
    assert_eq!(reloaded.blocked_until(), second.blocked_until());

    repository
        .clear_failed_attempts("throttled@example.com")
        .await
        .unwrap();
    assert_eq!(
        repository.failed_attempts("throttled@example.com").await.unwrap(),
        None
    );
}

#[tokio::test]
async fn zero_count_row_reads_as_no_attempts_and_clearing_is_idempotent() {
    let database = TestDatabase::open().await;
    let repository = database.repository();

    repository
        .clear_failed_attempts("never-seen@example.com")
        .await
        .unwrap();
    assert_eq!(
        repository.failed_attempts("never-seen@example.com").await.unwrap(),
        None
    );
    let _ = Email::parse("never-seen@example.com").unwrap();
    let _ = PasswordHash::from_stored("hash".to_owned());
}

#[tokio::test]
async fn clear_only_touches_the_target_identifier() {
    let database = TestDatabase::open().await;
    let repository = database.repository();

    repository
        .record_failed_attempt("a@example.com", at(0))
        .await
        .unwrap();
    repository
        .record_failed_attempt("b@example.com", at(0))
        .await
        .unwrap();

    repository.clear_failed_attempts("a@example.com").await.unwrap();

    assert_eq!(repository.failed_attempts("a@example.com").await.unwrap(), None);
    assert!(repository
        .failed_attempts("b@example.com")
        .await
        .unwrap()
        .is_some());
    let _ = RepositoryError::OperationFailed;
}
