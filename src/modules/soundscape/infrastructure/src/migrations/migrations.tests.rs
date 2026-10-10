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
        let pool = open_pool(&directory.path().join("soundscape.sqlite")).await;
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

async fn ledger_rows(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM soundscape_migrations")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn icon_column_count(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM pragma_table_info('soundscape_sounds') \
         WHERE name LIKE 'icon%'",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn up_is_idempotent_across_restarts() {
    let database = TestDatabase::open().await;
    migrate(&database.connection).await.unwrap();
    migrate(&database.connection).await.unwrap();
    assert_eq!(ledger_rows(&database.pool).await, 2);
    assert_eq!(icon_column_count(&database.pool).await, 3);
}

#[tokio::test]
async fn a_failing_migration_rolls_back_and_the_same_database_recovers() {
    use sea_orm_migration::MigratorTrait;

    // Base schema with only the base ledger row, then an obstruction on the
    // migration's SECOND statement (the second icon column), so the first
    // ALTER succeeds before the run fails. Without the transaction wrapper,
    // the first column would survive as partial schema.
    let database = TestDatabase::open().await;
    super::Migrator::up(&database.connection, Some(1))
        .await
        .unwrap();
    sqlx::query("ALTER TABLE soundscape_sounds ADD COLUMN icon_storage_key TEXT NULL")
        .execute(&database.pool)
        .await
        .unwrap();

    migrate(&database.connection).await.unwrap_err();

    assert_eq!(
        ledger_rows(&database.pool).await,
        1,
        "a failed run must not write ledger rows"
    );
    assert_eq!(
        icon_column_count(&database.pool).await,
        1,
        "only the pre-existing obstruction column may remain"
    );

    // Clear the obstruction: the same database now migrates cleanly.
    sqlx::query("ALTER TABLE soundscape_sounds DROP COLUMN icon_storage_key")
        .execute(&database.pool)
        .await
        .unwrap();
    migrate(&database.connection).await.unwrap();
    assert_eq!(ledger_rows(&database.pool).await, 2);
    assert_eq!(icon_column_count(&database.pool).await, 3);
}
