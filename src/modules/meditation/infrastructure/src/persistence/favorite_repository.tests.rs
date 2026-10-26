use chrono::{Duration, Utc};
use kernel::UserId;
use meditation_application::ports::FavoriteRepository;

use super::SeaOrmFavoriteRepository;
use std::collections::HashSet;
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

fn alice() -> UserId {
    UserId::new(ALICE)
}

fn bob() -> UserId {
    UserId::new(BOB)
}

#[tokio::test]
async fn add_is_idempotent_and_keeps_the_first_favorited_time() {
    let repository = SeaOrmFavoriteRepository::new(open().await);
    let now = Utc::now();
    repository.add(alice(), 10, now).await.unwrap();
    repository
        .add(alice(), 11, now + Duration::seconds(1))
        .await
        .unwrap();

    repository
        .add(alice(), 10, now + Duration::seconds(5))
        .await
        .unwrap();

    assert_eq!(
        repository.list_for_user(alice()).await.unwrap(),
        vec![11, 10]
    );
}

#[tokio::test]
async fn list_is_newest_first_then_highest_meditation_id() {
    let repository = SeaOrmFavoriteRepository::new(open().await);
    let now = Utc::now();
    repository.add(alice(), 1, now).await.unwrap();
    repository.add(alice(), 3, now).await.unwrap();
    repository
        .add(alice(), 2, now + Duration::seconds(1))
        .await
        .unwrap();

    assert_eq!(
        repository.list_for_user(alice()).await.unwrap(),
        vec![2, 3, 1]
    );
}

#[tokio::test]
async fn reads_are_scoped_to_the_user() {
    let repository = SeaOrmFavoriteRepository::new(open().await);
    repository.add(alice(), 1, Utc::now()).await.unwrap();
    repository.add(bob(), 2, Utc::now()).await.unwrap();

    assert_eq!(repository.list_for_user(alice()).await.unwrap(), vec![1]);
    assert_eq!(
        repository.ids_for_user(bob()).await.unwrap(),
        HashSet::from([2])
    );
}

#[tokio::test]
async fn remove_reports_whether_a_favorite_existed_and_spares_other_users() {
    let repository = SeaOrmFavoriteRepository::new(open().await);
    repository.add(alice(), 1, Utc::now()).await.unwrap();
    repository.add(bob(), 1, Utc::now()).await.unwrap();

    assert!(repository.remove(alice(), 1).await.unwrap());
    assert!(!repository.remove(alice(), 1).await.unwrap());
    assert_eq!(repository.list_for_user(bob()).await.unwrap(), vec![1]);
}

#[tokio::test]
async fn delete_for_meditation_removes_every_users_rows_for_it() {
    let repository = SeaOrmFavoriteRepository::new(open().await);
    repository.add(alice(), 1, Utc::now()).await.unwrap();
    repository.add(bob(), 1, Utc::now()).await.unwrap();
    repository.add(alice(), 2, Utc::now()).await.unwrap();

    assert_eq!(repository.delete_for_meditation(1).await.unwrap(), 2);

    assert_eq!(repository.list_for_user(alice()).await.unwrap(), vec![2]);
    assert!(repository.list_for_user(bob()).await.unwrap().is_empty());
}

#[tokio::test]
async fn delete_all_for_user_spares_other_users() {
    let repository = SeaOrmFavoriteRepository::new(open().await);
    repository.add(alice(), 1, Utc::now()).await.unwrap();
    repository.add(alice(), 2, Utc::now()).await.unwrap();
    repository.add(bob(), 1, Utc::now()).await.unwrap();

    assert_eq!(repository.delete_all_for_user(alice()).await.unwrap(), 2);

    assert!(repository.list_for_user(alice()).await.unwrap().is_empty());
    assert_eq!(repository.list_for_user(bob()).await.unwrap(), vec![1]);
}
