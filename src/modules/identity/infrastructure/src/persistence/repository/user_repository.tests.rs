use std::str::FromStr;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use identity_application::ports::{
    CredentialChange, MemberRegistration, RepositoryError, UserRepository,
};
use identity_domain::{AvatarRef, DisplayName, Email, PasswordHash, Role};
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
        let path = directory.path().join("user-repository.sqlite");
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

fn member_registration(address: &str) -> MemberRegistration {
    MemberRegistration::new(
        Email::parse(address).unwrap(),
        PasswordHash::from_stored(format!("hash-{address}")),
        Utc.with_ymd_and_hms(2026, 9, 26, 12, 0, 0).unwrap(),
    )
}

#[tokio::test]
async fn listing_is_ordered_by_email_and_finds_by_id() {
    let database = TestDatabase::open().await;
    let repository = database.repository();

    repository
        .insert_member(member_registration("zoe@example.com"))
        .await
        .unwrap();
    repository
        .insert_member(member_registration("adam@example.com"))
        .await
        .unwrap();

    let listed = repository.list().await.unwrap();
    let emails: Vec<&str> = listed.iter().map(|user| user.email().as_str()).collect();
    assert_eq!(emails, ["adam@example.com", "zoe@example.com"]);

    let by_id = repository
        .find_by_id(listed[1].id())
        .await
        .unwrap()
        .expect("seeded user must be findable by id");
    assert_eq!(by_id.email().as_str(), "zoe@example.com");

    let missing = repository.find_by_id(UserId::new(9999)).await.unwrap();
    assert_eq!(missing, None);
}

#[tokio::test]
async fn set_disabled_updates_only_the_target_and_rejects_unknown_ids() {
    let database = TestDatabase::open().await;
    let repository = database.repository();

    let kept = repository
        .insert_member(member_registration("kept@example.com"))
        .await
        .unwrap();
    let disabled = repository
        .insert_member(member_registration("disabled@example.com"))
        .await
        .unwrap();

    repository.set_disabled(disabled.id(), true).await.unwrap();

    let reloaded = repository
        .find_by_id(disabled.id())
        .await
        .unwrap()
        .expect("disabled user still exists");
    assert!(reloaded.is_disabled());

    let untouched = repository
        .find_by_id(kept.id())
        .await
        .unwrap()
        .expect("kept user still exists");
    assert!(!untouched.is_disabled());

    assert_eq!(
        repository.set_disabled(UserId::new(9999), true).await,
        Err(RepositoryError::UserNotFound)
    );
}

#[tokio::test]
async fn set_profile_stores_display_name_and_avatar() {
    let database = TestDatabase::open().await;
    let repository = database.repository();

    let member = repository
        .insert_member(member_registration("profiled@example.com"))
        .await
        .unwrap();

    let named = repository
        .set_profile(
            member.id(),
            Some(DisplayName::parse("Ahmet").unwrap()),
            Some(AvatarRef::new("avatar-0123456789abcdef.webp".to_owned(), "webp").unwrap()),
        )
        .await
        .unwrap();
    assert_eq!(
        named.display_name().map(|name| name.as_str()),
        Some("Ahmet")
    );
    assert!(named.avatar().is_some());

    let cleared = repository
        .set_profile(member.id(), None, None)
        .await
        .unwrap();
    assert_eq!(cleared.display_name(), None);
    assert_eq!(cleared.avatar(), None);

    assert_eq!(
        repository.set_profile(UserId::new(9999), None, None).await,
        Err(RepositoryError::UserNotFound)
    );
}

#[tokio::test]
async fn delete_user_removes_row_and_reports_unknown_ids() {
    let database = TestDatabase::open().await;
    let repository = database.repository();

    let member = repository
        .insert_member(member_registration("doomed@example.com"))
        .await
        .unwrap();

    assert!(repository.delete_user(member.id()).await.unwrap());
    assert_eq!(repository.find_by_id(member.id()).await.unwrap(), None);
    assert!(!repository.delete_user(member.id()).await.unwrap());
}

#[tokio::test]
async fn only_admins_count_as_active_admins() {
    let database = TestDatabase::open().await;
    let repository = database.repository();

    assert_eq!(repository.count_active_admins().await.unwrap(), 0);

    repository
        .insert_member(member_registration("member@example.com"))
        .await
        .unwrap();
    assert_eq!(repository.count_active_admins().await.unwrap(), 0);

    let member = repository
        .find_by_email(&Email::parse("member@example.com").unwrap())
        .await
        .unwrap()
        .unwrap();
    // Members can never hold the admin role through set_disabled; verify
    // the count through the role column via a direct restore path instead:
    // promote by applying a permanent credential change keeps role, so
    // insert an admin through the initial-admin port.
    use identity_application::ports::InitialAdminRegistration;
    let admin = repository
        .insert_initial_admin_if_uninitialized(InitialAdminRegistration::new(
            Email::parse("admin@example.com").unwrap(),
            PasswordHash::from_stored("hash-admin".to_owned()),
            Utc.with_ymd_and_hms(2026, 9, 26, 12, 0, 0).unwrap(),
        ))
        .await
        .unwrap();
    assert_eq!(admin.role(), Role::Admin);
    assert_eq!(repository.count_active_admins().await.unwrap(), 1);

    repository.set_disabled(admin.id(), true).await.unwrap();
    assert_eq!(repository.count_active_admins().await.unwrap(), 0);
    let _ = member;
}

#[tokio::test]
async fn credential_change_bumps_auth_version_and_clears_throttle() {
    let database = TestDatabase::open().await;
    let repository = database.repository();

    let member = repository
        .insert_member(member_registration("rotating@example.com"))
        .await
        .unwrap();
    use identity_application::ports::LoginThrottleRepository;
    use identity_domain::FailedAttempts;
    repository
        .record_failed_attempt("rotating@example.com", Utc::now())
        .await
        .unwrap();
    assert!(repository
        .failed_attempts("rotating@example.com")
        .await
        .unwrap()
        .is_some());

    let changed = repository
        .apply_credential_change(CredentialChange::permanent(
            member.id(),
            PasswordHash::from_stored("hash-rotated".to_owned()),
        ))
        .await
        .unwrap();

    assert_eq!(changed.auth_version(), 1);
    assert_eq!(changed.password_hash().as_str(), "hash-rotated");
    assert!(!changed.must_change_password());
    assert_eq!(
        repository
            .failed_attempts("rotating@example.com")
            .await
            .unwrap(),
        None,
        "credential change must clear the login throttle"
    );
    assert_eq!(
        repository
            .apply_credential_change(CredentialChange::permanent(
                UserId::new(9999),
                PasswordHash::from_stored("hash-nobody".to_owned()),
            ))
            .await,
        Err(RepositoryError::UserNotFound)
    );
    let _ = FailedAttempts::first_failure(Utc::now());
}
