use chrono::{Duration, Utc};
use meditation_application::ports::MembershipRepository;

use super::SeaOrmMembershipRepository;
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

#[tokio::test]
async fn add_appends_positions_and_is_idempotent_per_pair() {
    let repository = SeaOrmMembershipRepository::new(open().await);
    let now = Utc::now();

    let first = repository.add(1, 10, now).await.unwrap();
    let second = repository.add(1, 20, now).await.unwrap();
    let again = repository
        .add(1, 10, now + Duration::seconds(5))
        .await
        .unwrap();

    assert_ne!(first, second);
    assert_eq!(first, again);
    let listed = repository.list_by_program(1).await.unwrap();
    let pairs: Vec<(i64, i64)> = listed
        .iter()
        .map(|m| (m.meditation_id, m.position))
        .collect();
    assert_eq!(pairs, vec![(10, 0), (20, 1)]);
    assert_eq!(listed[0].program_id, 1);
    assert_eq!(listed[0].added_at.timestamp(), now.timestamp());
}

#[tokio::test]
async fn positions_are_independent_per_program() {
    let repository = SeaOrmMembershipRepository::new(open().await);
    let now = Utc::now();
    repository.add(1, 10, now).await.unwrap();
    repository.add(2, 10, now).await.unwrap();

    assert_eq!(repository.list_by_program(1).await.unwrap()[0].position, 0);
    assert_eq!(repository.list_by_program(2).await.unwrap()[0].position, 0);
    assert!(repository.list_by_program(3).await.unwrap().is_empty());
}

#[tokio::test]
async fn remove_reports_whether_the_pair_was_a_member() {
    let repository = SeaOrmMembershipRepository::new(open().await);
    repository.add(1, 10, Utc::now()).await.unwrap();

    assert!(!repository.remove(1, 99).await.unwrap());
    assert!(!repository.remove(2, 10).await.unwrap());
    assert!(repository.remove(1, 10).await.unwrap());
    assert!(repository.list_by_program(1).await.unwrap().is_empty());
}

#[tokio::test]
async fn set_order_rewrites_positions_and_ignores_unknown_ids() {
    let repository = SeaOrmMembershipRepository::new(open().await);
    let now = Utc::now();
    for meditation_id in [10, 20, 30] {
        repository.add(1, meditation_id, now).await.unwrap();
    }
    repository.add(2, 10, now).await.unwrap();

    repository.set_order(1, &[30, 999, 10, 20]).await.unwrap();

    let order: Vec<i64> = repository
        .list_by_program(1)
        .await
        .unwrap()
        .iter()
        .map(|m| m.meditation_id)
        .collect();
    assert_eq!(order, vec![30, 10, 20]);
    assert_eq!(repository.list_by_program(2).await.unwrap()[0].position, 0);
}

#[tokio::test]
async fn remove_for_meditation_counts_rows_across_programs() {
    let repository = SeaOrmMembershipRepository::new(open().await);
    let now = Utc::now();
    repository.add(1, 10, now).await.unwrap();
    repository.add(2, 10, now).await.unwrap();
    repository.add(2, 20, now).await.unwrap();

    assert_eq!(repository.remove_for_meditation(10).await.unwrap(), 2);
    assert_eq!(repository.remove_for_meditation(10).await.unwrap(), 0);
    assert_eq!(repository.list_by_program(2).await.unwrap().len(), 1);
}

#[tokio::test]
async fn first_program_for_meditation_is_the_earliest_added() {
    let repository = SeaOrmMembershipRepository::new(open().await);
    let now = Utc::now();
    repository.add(5, 10, now).await.unwrap();
    repository
        .add(3, 10, now - Duration::seconds(30))
        .await
        .unwrap();

    assert_eq!(
        repository.first_program_for_meditation(10).await.unwrap(),
        Some(3)
    );
    assert_eq!(
        repository.first_program_for_meditation(77).await.unwrap(),
        None
    );
}
