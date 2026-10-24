use chrono::{Duration, Utc};
use kernel::UserId;
use meditation_application::ports::SessionRepository;

use super::SeaOrmSessionRepository;
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
async fn record_if_new_stores_the_session() {
    let repository = SeaOrmSessionRepository::new(open().await);
    let started = Utc::now();
    let finished = started + Duration::minutes(10);

    let session = repository
        .record_if_new(UserId::new(ALICE), 10, started, finished)
        .await
        .unwrap();

    assert_eq!(session.user_id, UserId::new(ALICE));
    assert_eq!(session.meditation_id, 10);
    assert_eq!(session.started_at.timestamp(), started.timestamp());
    assert_eq!(session.finished_at.timestamp(), finished.timestamp());
}

#[tokio::test]
async fn record_if_new_is_idempotent_on_user_meditation_start() {
    let repository = SeaOrmSessionRepository::new(open().await);
    let started = Utc::now();

    let first = repository
        .record_if_new(
            UserId::new(ALICE),
            10,
            started,
            started + Duration::minutes(1),
        )
        .await
        .unwrap();
    let replay = repository
        .record_if_new(
            UserId::new(ALICE),
            10,
            started,
            started + Duration::minutes(9),
        )
        .await
        .unwrap();

    assert_eq!(first.id, replay.id);
    assert_eq!(replay.finished_at, first.finished_at);
    assert_eq!(
        repository.count_for(UserId::new(ALICE), 10).await.unwrap(),
        1
    );
}

#[tokio::test]
async fn distinct_starts_users_and_meditations_are_separate_sessions() {
    let repository = SeaOrmSessionRepository::new(open().await);
    let started = Utc::now();
    let finished = started + Duration::minutes(1);
    let base = repository
        .record_if_new(UserId::new(ALICE), 10, started, finished)
        .await
        .unwrap();

    let later = repository
        .record_if_new(
            UserId::new(ALICE),
            10,
            started + Duration::seconds(5),
            finished,
        )
        .await
        .unwrap();
    let other_user = repository
        .record_if_new(UserId::new(BOB), 10, started, finished)
        .await
        .unwrap();
    let other_meditation = repository
        .record_if_new(UserId::new(ALICE), 11, started, finished)
        .await
        .unwrap();

    let ids = [base.id, later.id, other_user.id, other_meditation.id];
    for (index, id) in ids.iter().enumerate() {
        assert!(!ids[index + 1..].contains(id));
    }
}

#[tokio::test]
async fn count_for_is_scoped_to_user_and_meditation() {
    let repository = SeaOrmSessionRepository::new(open().await);
    let started = Utc::now();
    let finished = started + Duration::minutes(1);
    for offset in 0..3 {
        repository
            .record_if_new(
                UserId::new(ALICE),
                10,
                started + Duration::seconds(offset),
                finished,
            )
            .await
            .unwrap();
    }
    repository
        .record_if_new(UserId::new(BOB), 10, started, finished)
        .await
        .unwrap();
    repository
        .record_if_new(UserId::new(ALICE), 11, started, finished)
        .await
        .unwrap();

    assert_eq!(
        repository.count_for(UserId::new(ALICE), 10).await.unwrap(),
        3
    );
    assert_eq!(repository.count_for(UserId::new(BOB), 10).await.unwrap(), 1);
    assert_eq!(repository.count_for(UserId::new(9), 10).await.unwrap(), 0);
}

#[tokio::test]
async fn list_for_user_returns_only_that_users_sessions() {
    let repository = SeaOrmSessionRepository::new(open().await);
    let started = Utc::now();
    let finished = started + Duration::minutes(1);
    repository
        .record_if_new(UserId::new(ALICE), 10, started, finished)
        .await
        .unwrap();
    repository
        .record_if_new(UserId::new(ALICE), 11, started, finished)
        .await
        .unwrap();
    repository
        .record_if_new(UserId::new(BOB), 10, started, finished)
        .await
        .unwrap();

    let sessions = repository.list_for_user(UserId::new(ALICE)).await.unwrap();

    assert_eq!(sessions.len(), 2);
    assert!(sessions
        .iter()
        .all(|session| session.user_id == UserId::new(ALICE)));
    assert!(repository
        .list_for_user(UserId::new(9))
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn delete_all_for_user_removes_only_that_users_sessions() {
    let repository = SeaOrmSessionRepository::new(open().await);
    let started = Utc::now();
    let finished = started + Duration::minutes(1);
    repository
        .record_if_new(UserId::new(ALICE), 10, started, finished)
        .await
        .unwrap();
    repository
        .record_if_new(UserId::new(ALICE), 11, started, finished)
        .await
        .unwrap();
    repository
        .record_if_new(UserId::new(BOB), 10, started, finished)
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
    assert_eq!(repository.count_for(UserId::new(BOB), 10).await.unwrap(), 1);
}
