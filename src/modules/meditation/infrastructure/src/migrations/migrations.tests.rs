use std::{path::Path, str::FromStr, time::Duration};

use sea_orm::{DatabaseConnection, SqlxSqliteConnector};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};

use super::migrate;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(5);

struct TestDatabase {
    _directory: tempfile::TempDir,
    pool: SqlitePool,
    connection: DatabaseConnection,
}

impl TestDatabase {
    async fn open() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let pool = open_pool(&directory.path().join("meditation.sqlite")).await;
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
        .foreign_keys(false)
        .busy_timeout(STARTUP_TIMEOUT);

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .unwrap()
}

async fn table_exists(pool: &SqlitePool, table: &str) -> bool {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
    )
    .bind(table)
    .fetch_one(pool)
    .await
    .unwrap()
        != 0
}

async fn ledger_rows(pool: &SqlitePool, table: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM {table}"))
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn up_creates_every_module_table_under_its_own_ledger() {
    let database = TestDatabase::open().await;
    migrate(&database.connection).await.unwrap();

    for table in [
        "meditation_meditations",
        "meditation_programs",
        "meditation_categories",
        "meditation_program_memberships",
        "meditation_playback_state",
        "meditation_sessions",
        "meditation_notes",
    ] {
        assert!(
            table_exists(&database.pool, table).await,
            "{table} should exist after up()"
        );
    }
    assert_eq!(
        ledger_rows(&database.pool, "meditation_migrations").await,
        2
    );
    // Ledger isolation: identity's ledger is untouched.
    assert!(!table_exists(&database.pool, "identity_migrations").await);
}

#[tokio::test]
async fn up_is_idempotent_across_restarts() {
    let database = TestDatabase::open().await;
    migrate(&database.connection).await.unwrap();
    migrate(&database.connection).await.unwrap();
    assert_eq!(
        ledger_rows(&database.pool, "meditation_migrations").await,
        2
    );
}

#[tokio::test]
async fn down_is_refused_and_leaves_schema_untouched() {
    use sea_orm_migration::MigratorTrait;

    let database = TestDatabase::open().await;
    migrate(&database.connection).await.unwrap();

    let before = ledger_rows(&database.pool, "meditation_meditations").await;
    let result = super::Migrator::down(&database.connection, None).await;
    assert!(result.is_err(), "down() must refuse to run");

    let after = ledger_rows(&database.pool, "meditation_meditations").await;
    assert_eq!(before, after);
    // The refused down() did not roll the applied migration back either.
    assert_eq!(
        ledger_rows(&database.pool, "meditation_migrations").await,
        2
    );
}

async fn column_names(pool: &SqlitePool, table: &str) -> Vec<String> {
    sqlx::query_scalar::<_, String>(&format!("SELECT name FROM pragma_table_info('{table}')"))
        .fetch_all(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn a_failing_migration_rolls_back_and_the_same_database_recovers() {
    use sea_orm_migration::MigratorTrait;

    // Base schema with only the base ledger row — the state of a database on
    // the shipped schema — then an obstruction on the migration's SECOND
    // statement, so the first (adding `description`) succeeds before the run
    // fails. Without the transaction wrapper, `description` would survive.
    let database = TestDatabase::open().await;
    super::Migrator::up(&database.connection, Some(1))
        .await
        .unwrap();
    sqlx::query(
        "ALTER TABLE meditation_program_memberships \
         ADD COLUMN position INTEGER NOT NULL DEFAULT 0",
    )
    .execute(&database.pool)
    .await
    .unwrap();

    migrate(&database.connection).await.unwrap_err();

    assert_eq!(
        ledger_rows(&database.pool, "meditation_migrations").await,
        1,
        "a failed run must not write ledger rows"
    );
    let described = column_names(&database.pool, "meditation_meditations").await;
    assert!(
        !described.contains(&"description".to_owned()),
        "the first statement's column must be rolled back: {described:?}"
    );
    let positioned = column_names(&database.pool, "meditation_program_memberships").await;
    assert!(
        positioned.contains(&"position".to_owned()),
        "the pre-existing obstruction is untouched"
    );

    // Clear the obstruction: the same database now migrates cleanly.
    sqlx::query("ALTER TABLE meditation_program_memberships DROP COLUMN position")
        .execute(&database.pool)
        .await
        .unwrap();
    migrate(&database.connection).await.unwrap();
    assert_eq!(
        ledger_rows(&database.pool, "meditation_migrations").await,
        2
    );
    let described = column_names(&database.pool, "meditation_meditations").await;
    assert!(described.contains(&"description".to_owned()));
}
