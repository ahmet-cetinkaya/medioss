use std::{path::Path, str::FromStr, time::Duration};

use sea_orm::{DatabaseConnection, SqlxSqliteConnector};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};
use tower_sessions_sqlx_store::SqliteStore;

use super::migrate;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(5);
const USER_INSERT: &str = r#"
    INSERT INTO users (
        email,
        password_hash,
        role,
        password_expires_at,
        created_at
    ) VALUES (?, 'password-hash', 'admin', NULL, '2026-09-08T00:00:00Z')
"#;

struct TestDatabase {
    _directory: tempfile::TempDir,
    pool: SqlitePool,
    connection: DatabaseConnection,
}

impl TestDatabase {
    async fn open() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let pool = open_pool(&directory.path().join("identity.sqlite")).await;
        let connection = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool.clone());

        Self {
            _directory: directory,
            pool,
            connection,
        }
    }
}

async fn open_pool(path: &Path) -> SqlitePool {
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
        .unwrap()
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .busy_timeout(STARTUP_TIMEOUT);

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .unwrap()
}

async fn run_identity_then_session_migrations(
    connection: &DatabaseConnection,
    pool: &SqlitePool,
) -> Result<(), String> {
    migrate(connection)
        .await
        .map_err(|error| format!("identity migration failed: {error}"))?;
    SqliteStore::new(pool.clone())
        .migrate()
        .await
        .map_err(|error| format!("session migration failed: {error}"))
}

async fn table_exists(pool: &SqlitePool, table: &str) -> bool {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
    )
    .bind(table)
    .fetch_one(pool)
    .await
    .unwrap()
        == 1
}

#[tokio::test]
async fn migration_is_idempotent_and_uses_a_globally_namespaced_history_entry() {
    let database = TestDatabase::open().await;

    migrate(&database.connection).await.unwrap();
    migrate(&database.connection).await.unwrap();

    let applied: Vec<String> =
        sqlx::query_scalar("SELECT version FROM seaql_migrations ORDER BY version")
            .fetch_all(&database.pool)
            .await
            .unwrap();
    assert_eq!(
        applied,
        [
            "m20260908_000001_identity_create_schema",
            "m20261002_000001_identity_profile_columns",
            "m20261012_000001_identity_avatar_key",
        ]
    );
}

#[tokio::test]
async fn required_identity_tables_and_singleton_state_are_initialized() {
    let database = TestDatabase::open().await;
    migrate(&database.connection).await.unwrap();

    for table in ["users", "instance_state", "login_attempts"] {
        assert!(table_exists(&database.pool, table).await, "missing {table}");
    }

    let state_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM instance_state")
        .fetch_one(&database.pool)
        .await
        .unwrap();
    assert_eq!(state_rows, 1);
    assert!(
        sqlx::query("INSERT INTO instance_state (singleton) VALUES (1)")
            .execute(&database.pool)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("INSERT INTO instance_state (singleton) VALUES (2)")
            .execute(&database.pool)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn must_change_password_schema_default_check_and_values_are_persistent() {
    let database = TestDatabase::open().await;
    migrate(&database.connection).await.unwrap();

    let columns: Vec<(String, i64, Option<String>)> =
        sqlx::query_as("SELECT name, \"notnull\", dflt_value FROM pragma_table_info('users')")
            .fetch_all(&database.pool)
            .await
            .unwrap();
    let must_change_password = columns
        .iter()
        .find(|(name, _, _)| name == "must_change_password")
        .unwrap();
    assert_eq!(must_change_password.1, 1);
    assert_eq!(must_change_password.2.as_deref(), Some("0"));

    let default_user_id = insert_user(&database.pool, "[EMAIL_REDACTED]")
        .await
        .unwrap();
    let must_change: i64 =
        sqlx::query_scalar("SELECT must_change_password FROM users WHERE id = ?")
            .bind(default_user_id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(must_change, 0);

    sqlx::query("UPDATE users SET must_change_password = 1 WHERE id = ?")
        .bind(default_user_id)
        .execute(&database.pool)
        .await
        .unwrap();
    let persisted_true: i64 =
        sqlx::query_scalar("SELECT must_change_password FROM users WHERE id = ?")
            .bind(default_user_id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(persisted_true, 1);

    sqlx::query("UPDATE users SET must_change_password = 0 WHERE id = ?")
        .bind(default_user_id)
        .execute(&database.pool)
        .await
        .unwrap();
    let persisted_false: i64 =
        sqlx::query_scalar("SELECT must_change_password FROM users WHERE id = ?")
            .bind(default_user_id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(persisted_false, 0);

    assert!(
        sqlx::query("UPDATE users SET must_change_password = 2 WHERE id = ?")
            .bind(default_user_id)
            .execute(&database.pool)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn users_require_normalized_case_insensitively_unique_emails() {
    let database = TestDatabase::open().await;
    migrate(&database.connection).await.unwrap();

    insert_user(&database.pool, "[EMAIL_REDACTED]")
        .await
        .unwrap();
    assert!(insert_user(&database.pool, "[EMAIL_REDACTED]")
        .await
        .is_err());
    assert!(insert_user(&database.pool, "[EMAIL_REDACTED]")
        .await
        .is_err());
}

#[tokio::test]
async fn login_attempts_accept_unknown_normalized_identifiers_without_user_rows() {
    let database = TestDatabase::open().await;
    migrate(&database.connection).await.unwrap();

    sqlx::query(
        "INSERT INTO login_attempts \
         (normalized_identifier, failed_attempt_count, last_failed_attempt_at, blocked_until) \
         VALUES ('missing-identifier', 1, '2026-09-08T00:00:00Z', '2026-09-08T00:00:01Z')",
    )
    .execute(&database.pool)
    .await
    .unwrap();

    let attempts: i64 = sqlx::query_scalar(
        "SELECT failed_attempt_count FROM login_attempts \
         WHERE normalized_identifier = 'missing-identifier'",
    )
    .fetch_one(&database.pool)
    .await
    .unwrap();
    assert_eq!(attempts, 1);
}

#[tokio::test]
async fn concurrent_initialization_claims_produce_exactly_one_admin() {
    let database = TestDatabase::open().await;
    migrate(&database.connection).await.unwrap();

    let first_pool = database.pool.clone();
    let second_pool = database.pool.clone();
    let first = tokio::spawn(async move {
        let mut transaction = first_pool.begin_with("BEGIN IMMEDIATE").await?;
        let claimed = sqlx::query("DELETE FROM instance_state WHERE singleton = 1")
            .execute(&mut *transaction)
            .await?
            .rows_affected();
        let result = if claimed == 1 {
            insert_user(&mut *transaction, "[EMAIL_REDACTED]").await?;
            true
        } else {
            false
        };
        transaction.commit().await?;
        Ok::<_, sqlx::Error>(result)
    });
    let second = tokio::spawn(async move {
        let mut transaction = second_pool.begin_with("BEGIN IMMEDIATE").await?;
        let claimed = sqlx::query("DELETE FROM instance_state WHERE singleton = 1")
            .execute(&mut *transaction)
            .await?
            .rows_affected();
        let result = if claimed == 1 {
            insert_user(&mut *transaction, "[EMAIL_REDACTED]").await?;
            true
        } else {
            false
        };
        transaction.commit().await?;
        Ok::<_, sqlx::Error>(result)
    });

    let outcomes = tokio::time::timeout(STARTUP_TIMEOUT, async {
        [
            first.await.unwrap().unwrap(),
            second.await.unwrap().unwrap(),
        ]
    })
    .await
    .unwrap();
    assert_eq!(outcomes.into_iter().filter(|claimed| *claimed).count(), 1);

    let users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&database.pool)
        .await
        .unwrap();
    assert_eq!(users, 1);
}

async fn insert_user<'e, E>(executor: E, email: &str) -> Result<i64, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    sqlx::query(USER_INSERT)
        .bind(email)
        .execute(executor)
        .await
        .map(|result| result.last_insert_rowid())
}

#[tokio::test]
async fn independent_pools_can_run_identity_then_session_startup_migrations() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("startup.sqlite");
    let first_pool = open_pool(&path).await;
    let second_pool = open_pool(&path).await;
    let first_connection = SqlxSqliteConnector::from_sqlx_sqlite_pool(first_pool.clone());
    let second_connection = SqlxSqliteConnector::from_sqlx_sqlite_pool(second_pool.clone());

    let first = run_identity_then_session_migrations(&first_connection, &first_pool);
    let second = run_identity_then_session_migrations(&second_connection, &second_pool);
    let (first_result, second_result) =
        tokio::time::timeout(STARTUP_TIMEOUT, async { tokio::join!(first, second) })
            .await
            .unwrap();
    let successes = [first_result, second_result]
        .into_iter()
        .filter(Result::is_ok)
        .count();
    assert!(successes >= 1);

    run_identity_then_session_migrations(&first_connection, &first_pool)
        .await
        .unwrap();
    for table in [
        "seaql_migrations",
        "users",
        "instance_state",
        "login_attempts",
        "tower_sessions",
    ] {
        assert!(table_exists(&first_pool, table).await, "missing {table}");
    }
}
