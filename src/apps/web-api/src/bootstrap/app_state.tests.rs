use super::build_state;
use crate::{
    bootstrap::{database, migrations::migrate_all},
    test_support::test_config,
};

#[tokio::test]
async fn build_state_wires_the_config_and_migrates_the_session_store() {
    let data = tempfile::tempdir().unwrap();
    let database = database::open(&data.path().join("medioss.db"))
        .await
        .unwrap();
    migrate_all(&database).await;
    let pool = database.pool.clone();

    let state = build_state(test_config(data.path(), false), database).await;

    assert_eq!(state.config.medioss_origin, "https://medioss.example");
    assert_eq!(state.config.data_dir, data.path());
    let session_tables: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'tower_sessions'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(session_tables, 1);
}
