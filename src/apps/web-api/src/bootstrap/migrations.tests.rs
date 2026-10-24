use super::migrate_all;
use crate::bootstrap::database;

const LEDGER_TABLES: [&str; 3] = [
    "meditation_migrations",
    "soundscape_migrations",
    "mood_tracker_migrations",
];

async fn table_names(pool: &sqlx::SqlitePool) -> Vec<String> {
    sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table'")
        .fetch_all(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn migrate_all_creates_every_module_ledger_and_is_idempotent() {
    let directory = tempfile::tempdir().unwrap();
    let database = database::open(&directory.path().join("medioss.db"))
        .await
        .unwrap();

    migrate_all(&database).await;
    let first = table_names(&database.pool).await;
    migrate_all(&database).await;
    let second = table_names(&database.pool).await;

    for ledger in LEDGER_TABLES {
        assert!(first.iter().any(|name| name == ledger), "missing {ledger}");
    }
    assert!(first.len() > LEDGER_TABLES.len());
    assert_eq!(first.len(), second.len());
}
