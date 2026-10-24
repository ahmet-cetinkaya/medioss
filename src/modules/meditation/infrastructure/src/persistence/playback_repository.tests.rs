use kernel::UserId;
use meditation_application::ports::PlaybackStateRepository;

use super::SeaOrmPlaybackRepository;
use std::time::Duration as StdDuration;

use sea_orm::{DatabaseConnection, SqlxSqliteConnector};
use sqlx::sqlite::SqlitePoolOptions;

use crate::migrations::migrate;

async fn open() -> DatabaseConnection {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .idle_timeout(None::<StdDuration>)
        .max_lifetime(None::<StdDuration>)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    let connection = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
    migrate(&connection).await.unwrap();
    connection
}

const ALICE: i64 = 1;
const BOB: i64 = 2;

#[tokio::test]
async fn get_of_untouched_state_is_none() {
    let repository = SeaOrmPlaybackRepository::new(open().await);

    assert!(repository
        .get(UserId::new(ALICE), 10)
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn upsert_position_creates_then_updates_keeping_completion() {
    let repository = SeaOrmPlaybackRepository::new(open().await);
    let alice = UserId::new(ALICE);

    repository.upsert_position(alice, 10, 1500).await.unwrap();
    let created = repository.get(alice, 10).await.unwrap().unwrap();
    assert_eq!((created.position_ms, created.completed), (1500, false));

    repository.set_completed(alice, 10, true).await.unwrap();
    repository.upsert_position(alice, 10, 3000).await.unwrap();
    let updated = repository.get(alice, 10).await.unwrap().unwrap();
    assert_eq!((updated.position_ms, updated.completed), (3000, true));
}

#[tokio::test]
async fn set_completed_on_untouched_row_starts_at_zero_and_can_unset() {
    let repository = SeaOrmPlaybackRepository::new(open().await);
    let alice = UserId::new(ALICE);

    repository.set_completed(alice, 10, true).await.unwrap();
    let state = repository.get(alice, 10).await.unwrap().unwrap();
    assert_eq!((state.position_ms, state.completed), (0, true));

    repository.set_completed(alice, 10, false).await.unwrap();
    assert!(!repository.get(alice, 10).await.unwrap().unwrap().completed);
}

#[tokio::test]
async fn reset_position_zeroes_position_but_keeps_completion() {
    let repository = SeaOrmPlaybackRepository::new(open().await);
    let alice = UserId::new(ALICE);
    repository.upsert_position(alice, 10, 900).await.unwrap();
    repository.set_completed(alice, 10, true).await.unwrap();

    repository.reset_position(alice, 10).await.unwrap();

    let state = repository.get(alice, 10).await.unwrap().unwrap();
    assert_eq!((state.position_ms, state.completed), (0, true));
}

#[tokio::test]
async fn state_is_scoped_per_user_and_meditation() {
    let repository = SeaOrmPlaybackRepository::new(open().await);
    repository
        .upsert_position(UserId::new(ALICE), 10, 111)
        .await
        .unwrap();
    repository
        .upsert_position(UserId::new(BOB), 10, 222)
        .await
        .unwrap();
    repository
        .upsert_position(UserId::new(ALICE), 11, 333)
        .await
        .unwrap();

    assert_eq!(
        repository
            .get(UserId::new(ALICE), 10)
            .await
            .unwrap()
            .unwrap()
            .position_ms,
        111
    );
    assert_eq!(
        repository
            .get(UserId::new(BOB), 10)
            .await
            .unwrap()
            .unwrap()
            .position_ms,
        222
    );
    assert_eq!(
        repository
            .get(UserId::new(ALICE), 11)
            .await
            .unwrap()
            .unwrap()
            .position_ms,
        333
    );
}

#[tokio::test]
async fn list_touched_returns_only_the_users_meditations_most_recent_first() {
    let repository = SeaOrmPlaybackRepository::new(open().await);
    let alice = UserId::new(ALICE);
    repository.upsert_position(alice, 10, 1).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    repository.upsert_position(alice, 20, 1).await.unwrap();
    repository
        .upsert_position(UserId::new(BOB), 30, 1)
        .await
        .unwrap();

    let touched = repository.list_touched(alice).await.unwrap();

    let ids: Vec<i64> = touched.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, vec![20, 10]);
    assert!(touched[0].1 >= touched[1].1);
    assert!(repository
        .list_touched(UserId::new(9))
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn delete_all_for_user_removes_only_that_users_rows() {
    let repository = SeaOrmPlaybackRepository::new(open().await);
    repository
        .upsert_position(UserId::new(ALICE), 10, 1)
        .await
        .unwrap();
    repository
        .upsert_position(UserId::new(ALICE), 11, 1)
        .await
        .unwrap();
    repository
        .upsert_position(UserId::new(BOB), 10, 1)
        .await
        .unwrap();

    assert_eq!(
        repository
            .delete_all_for_user(UserId::new(ALICE))
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        repository
            .delete_all_for_user(UserId::new(ALICE))
            .await
            .unwrap(),
        0
    );
    assert!(repository
        .get(UserId::new(BOB), 10)
        .await
        .unwrap()
        .is_some());
}
