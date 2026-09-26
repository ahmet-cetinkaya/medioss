use std::{str::FromStr, time::Duration};

use chrono::{TimeZone, Utc};
use identity_application::ports::{
    CredentialChange, InitialAdminRegistration, LoginThrottleRepository, MemberRegistration,
    RepositoryError, UserRepository,
};
use identity_domain::{Email, PasswordHash, Role};
use kernel::UserId;
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
        let path = directory.path().join("identity.sqlite");
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

fn email(value: &str) -> Email {
    Email::parse(value).unwrap()
}

fn hash(value: &str) -> PasswordHash {
    PasswordHash::from_stored(format!("hash-{value}"))
}

fn admin_registration(address: &str) -> InitialAdminRegistration {
    InitialAdminRegistration::new(
        email(address),
        hash("admin-password"),
        Utc.with_ymd_and_hms(2026, 9, 26, 12, 0, 0).unwrap(),
    )
}

fn member_registration(address: &str) -> MemberRegistration {
    MemberRegistration::new(
        email(address),
        hash("member-password"),
        Utc.with_ymd_and_hms(2026, 9, 26, 12, 0, 0).unwrap(),
    )
}

#[tokio::test]
async fn initial_admin_is_created_once_then_setup_is_rejected() {
    let database = TestDatabase::open().await;
    let repository = database.repository();

    let created = repository
        .insert_initial_admin_if_uninitialized(admin_registration("Root@Example.com"))
        .await
        .unwrap();

    assert_eq!(created.role(), Role::Admin);
    assert_eq!(created.email().as_str(), "root@example.com");
    assert_eq!(created.auth_version(), 0);
    assert!(!created.must_change_password());
    assert_eq!(created.password_expires_at(), None);

    let second = repository
        .insert_initial_admin_if_uninitialized(admin_registration("other@example.com"))
        .await
        .unwrap_err();
    assert_eq!(second, RepositoryError::SetupAlreadyCompleted);

    assert_eq!(repository.count().await.unwrap(), 1);
}

#[tokio::test]
async fn member_insertion_enforces_unique_email_and_roundtrips() {
    let database = TestDatabase::open().await;
    let repository = database.repository();

    let member = repository
        .insert_member(member_registration("Member@Example.com"))
        .await
        .unwrap();
    assert_eq!(member.role(), Role::Member);
    assert!(member.must_change_password());
    assert!(member.password_expires_at().is_some());

    let duplicate = repository
        .insert_member(member_registration("member@example.com"))
        .await
        .unwrap_err();
    assert_eq!(duplicate, RepositoryError::EmailAlreadyExists);

    let found = repository
        .find_by_email(&email("MEMBER@example.com"))
        .await
        .unwrap()
        .expect("member must be findable case-insensitively");
    assert_eq!(found.id(), member.id());
    assert_eq!(found.password_hash().as_str(), "hash-member-password");
}

#[tokio::test]
async fn credential_change_updates_user_and_clears_throttle_atomically() {
    let database = TestDatabase::open().await;
    let repository = database.repository();

    let member = repository
        .insert_member(member_registration("member@example.com"))
        .await
        .unwrap();
    repository
        .record_failed_attempt("member@example.com", Utc::now())
        .await
        .unwrap();

    let changed = repository
        .apply_credential_change(CredentialChange::permanent(member.id(), hash("brand-new")))
        .await
        .unwrap();

    assert_eq!(changed.auth_version(), member.auth_version() + 1);
    assert!(!changed.must_change_password());
    assert_eq!(changed.password_expires_at(), None);
    assert_eq!(changed.password_hash().as_str(), "hash-brand-new");

    let reloaded = repository
        .find_by_email(&email("member@example.com"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(reloaded.auth_version(), changed.auth_version());

    let throttle = repository
        .failed_attempts("member@example.com")
        .await
        .unwrap();
    assert!(throttle.is_none());
}

#[tokio::test]
async fn credential_change_for_missing_user_reports_user_not_found() {
    let database = TestDatabase::open().await;
    let repository = database.repository();

    let error = repository
        .apply_credential_change(CredentialChange::permanent(
            UserId::new(4242),
            hash("irrelevant"),
        ))
        .await
        .unwrap_err();
    assert_eq!(error, RepositoryError::UserNotFound);
}

#[tokio::test]
async fn failed_attempts_record_grows_and_clears_for_unknown_identifiers() {
    let database = TestDatabase::open().await;
    let repository = database.repository();
    let identifier = "ghost@example.com";

    let first = repository
        .record_failed_attempt(identifier, Utc::now())
        .await
        .unwrap();
    assert_eq!(first.count(), 1);

    let second = repository
        .record_failed_attempt(identifier, Utc::now() + chrono::TimeDelta::seconds(2))
        .await
        .unwrap();
    assert_eq!(second.count(), 2);
    assert!(second.blocked_until() > first.blocked_until());

    let stored = repository
        .failed_attempts(identifier)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.count(), 2);

    repository.clear_failed_attempts(identifier).await.unwrap();
    assert!(repository
        .failed_attempts(identifier)
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn corrupt_role_value_surfaces_typed_error_not_panic() {
    let database = TestDatabase::open().await;
    let repository = database.repository();
    repository
        .insert_member(member_registration("victim@example.com"))
        .await
        .unwrap();

    sea_orm::ConnectionTrait::execute_unprepared(
        &database.connection,
        "UPDATE users SET role = 'warlord' WHERE email = 'victim@example.com'",
    )
    .await
    .unwrap();

    let error = repository
        .find_by_email(&email("victim@example.com"))
        .await
        .unwrap_err();
    assert_eq!(error, RepositoryError::OperationFailed);
}

#[tokio::test]
async fn restored_user_rejects_must_change_without_expiry() {
    let database = TestDatabase::open().await;
    let repository = database.repository();
    repository
        .insert_member(member_registration("weird@example.com"))
        .await
        .unwrap();

    sea_orm::ConnectionTrait::execute_unprepared(
        &database.connection,
        "UPDATE users SET password_expires_at = NULL WHERE email = 'weird@example.com'",
    )
    .await
    .unwrap();

    let error = repository
        .find_by_email(&email("weird@example.com"))
        .await
        .unwrap_err();
    assert_eq!(error, RepositoryError::OperationFailed);
}
