use chrono::{DateTime, Duration, Utc};
use meditation_application::ports::MeditationRepository;

use super::SeaOrmMeditationRepository;
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

async fn add(
    repository: &SeaOrmMeditationRepository,
    name: &str,
    key: &str,
    created_at: DateTime<Utc>,
) -> i64 {
    repository
        .insert(
            name.to_owned(),
            "audio/ogg",
            10,
            key.to_owned(),
            5,
            created_at,
            None,
        )
        .await
        .unwrap()
        .id()
}

#[tokio::test]
async fn insert_round_trips_every_field() {
    let repository = SeaOrmMeditationRepository::new(open().await);
    let created_at = Utc::now();

    let stored = repository
        .insert(
            "Calm".to_owned(),
            "audio/ogg",
            1234,
            "folder/calm".to_owned(),
            6000,
            created_at,
            Some("Slow breathing".to_owned()),
        )
        .await
        .unwrap();
    let found = repository.find_by_id(stored.id()).await.unwrap().unwrap();

    assert_eq!(found.name(), "Calm");
    assert_eq!(found.content_type(), "audio/ogg");
    assert_eq!(found.size_bytes(), 1234);
    assert_eq!(found.storage_key(), "folder/calm");
    assert_eq!(found.duration_ms(), 6000);
    assert_eq!(found.description(), Some("Slow breathing"));
    assert_eq!(found.created_at().timestamp(), created_at.timestamp());
}

#[tokio::test]
async fn find_by_id_of_unknown_row_is_none() {
    let repository = SeaOrmMeditationRepository::new(open().await);

    assert!(repository.find_by_id(7).await.unwrap().is_none());
}

#[tokio::test]
async fn insert_rejects_values_the_domain_refuses() {
    let repository = SeaOrmMeditationRepository::new(open().await);

    let error = repository
        .insert(
            String::new(),
            "audio/ogg",
            10,
            "k".to_owned(),
            5,
            Utc::now(),
            None,
        )
        .await;

    assert!(error.is_err());
}

#[tokio::test]
async fn list_all_orders_by_created_at_then_id_descending() {
    let repository = SeaOrmMeditationRepository::new(open().await);
    let now = Utc::now();
    let oldest = add(&repository, "Oldest", "k1", now - Duration::seconds(10)).await;
    let tie_low = add(&repository, "TieLow", "k2", now).await;
    let tie_high = add(&repository, "TieHigh", "k3", now).await;

    let ids: Vec<i64> = repository
        .list_all()
        .await
        .unwrap()
        .iter()
        .map(|meditation| meditation.id())
        .collect();

    assert_eq!(ids, vec![tie_high, tie_low, oldest]);
}

#[tokio::test]
async fn storage_prefix_listing_matches_prefix_and_escapes_wildcards() {
    let repository = SeaOrmMeditationRepository::new(open().await);
    let now = Utc::now();
    add(&repository, "A", "prog-1/a", now).await;
    add(&repository, "B", "prog-1/b", now).await;
    add(&repository, "C", "prog-2/c", now).await;
    add(&repository, "D", "progX1/d", now).await;

    let names = |rows: Vec<meditation_domain::Meditation>| {
        let mut names: Vec<String> = rows.iter().map(|row| row.name().to_owned()).collect();
        names.sort();
        names
    };

    assert_eq!(
        names(repository.list_by_storage_prefix("prog-1/").await.unwrap()),
        vec!["A", "B"]
    );
    assert!(repository
        .list_by_storage_prefix("nothing")
        .await
        .unwrap()
        .is_empty());
    assert!(repository
        .list_by_storage_prefix("prog%")
        .await
        .unwrap()
        .is_empty());
    assert!(repository
        .list_by_storage_prefix("prog_1")
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn set_description_sets_clears_and_reports_missing() {
    let repository = SeaOrmMeditationRepository::new(open().await);
    let id = add(&repository, "A", "k", Utc::now()).await;

    assert!(repository.set_description(id, Some("Text")).await.unwrap());
    assert_eq!(
        repository
            .find_by_id(id)
            .await
            .unwrap()
            .unwrap()
            .description(),
        Some("Text")
    );
    assert!(repository.set_description(id, None).await.unwrap());
    assert_eq!(
        repository
            .find_by_id(id)
            .await
            .unwrap()
            .unwrap()
            .description(),
        None
    );
    assert!(!repository.set_description(999, Some("x")).await.unwrap());
}

#[tokio::test]
async fn set_storage_key_relocates_or_errors_for_missing_row() {
    let repository = SeaOrmMeditationRepository::new(open().await);
    let id = add(&repository, "A", "old", Utc::now()).await;

    repository.set_storage_key(id, "new/key").await.unwrap();
    assert_eq!(
        repository
            .find_by_id(id)
            .await
            .unwrap()
            .unwrap()
            .storage_key(),
        "new/key"
    );

    let error = repository
        .set_storage_key(999, "x")
        .await
        .expect_err("missing");
    assert!(error.contains("999"));
}

#[tokio::test]
async fn delete_reports_whether_a_row_was_removed() {
    let repository = SeaOrmMeditationRepository::new(open().await);
    let id = add(&repository, "A", "k", Utc::now()).await;

    assert!(repository.delete(id).await.unwrap());
    assert!(!repository.delete(id).await.unwrap());
    assert!(repository.list_all().await.unwrap().is_empty());
}
