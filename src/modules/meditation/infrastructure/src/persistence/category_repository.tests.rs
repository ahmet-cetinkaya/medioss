use chrono::{Duration, Utc};
use meditation_application::ports::{CategoryRepository, RenameCategoryError};

use super::SeaOrmCategoryRepository;
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
async fn insert_assigns_ids_and_finds_by_name_and_id() {
    let repository = SeaOrmCategoryRepository::new(open().await);
    let created_at = Utc::now();

    let first = repository
        .insert("Morning".to_owned(), created_at)
        .await
        .unwrap();
    let second = repository
        .insert("Evening".to_owned(), created_at)
        .await
        .unwrap();

    assert_ne!(first.id, second.id);
    assert_eq!(first.name, "Morning");
    assert_eq!(first.created_at.timestamp(), created_at.timestamp());
    assert_eq!(
        repository
            .find_by_name("Evening")
            .await
            .unwrap()
            .unwrap()
            .id,
        second.id
    );
    assert_eq!(
        repository.find_by_id(first.id).await.unwrap().unwrap().name,
        "Morning"
    );
}

#[tokio::test]
async fn lookups_of_unknown_categories_return_none() {
    let repository = SeaOrmCategoryRepository::new(open().await);

    assert!(repository.find_by_id(42).await.unwrap().is_none());
    assert!(repository.find_by_name("Missing").await.unwrap().is_none());
    assert!(repository.list_all().await.unwrap().is_empty());
}

#[tokio::test]
async fn find_by_name_is_case_sensitive_exact_match() {
    let repository = SeaOrmCategoryRepository::new(open().await);
    repository
        .insert("Morning".to_owned(), Utc::now())
        .await
        .unwrap();

    assert!(repository.find_by_name("morning").await.unwrap().is_none());
    assert!(repository.find_by_name("Morn").await.unwrap().is_none());
}

#[tokio::test]
async fn list_all_returns_every_category() {
    let repository = SeaOrmCategoryRepository::new(open().await);
    repository.insert("A".to_owned(), Utc::now()).await.unwrap();
    repository
        .insert("B".to_owned(), Utc::now() + Duration::seconds(1))
        .await
        .unwrap();

    let mut names: Vec<String> = repository
        .list_all()
        .await
        .unwrap()
        .into_iter()
        .map(|category| category.name)
        .collect();
    names.sort();

    assert_eq!(names, vec!["A".to_owned(), "B".to_owned()]);
}

#[tokio::test]
async fn duplicate_insert_is_rejected() {
    let repository = SeaOrmCategoryRepository::new(open().await);
    repository
        .insert("Morning".to_owned(), Utc::now())
        .await
        .unwrap();

    let error = repository
        .insert("Morning".to_owned(), Utc::now())
        .await
        .expect_err("unique name");

    assert!(error.contains("UNIQUE"));
}

#[tokio::test]
async fn rename_updates_name_and_reports_missing_and_duplicate() {
    let repository = SeaOrmCategoryRepository::new(open().await);
    let morning = repository
        .insert("Morning".to_owned(), Utc::now())
        .await
        .unwrap();
    repository
        .insert("Evening".to_owned(), Utc::now())
        .await
        .unwrap();

    assert!(repository
        .rename(morning.id, "Dawn".to_owned(), Utc::now())
        .await
        .unwrap());
    assert_eq!(
        repository
            .find_by_id(morning.id)
            .await
            .unwrap()
            .unwrap()
            .name,
        "Dawn"
    );
    assert!(!repository
        .rename(999, "Nope".to_owned(), Utc::now())
        .await
        .unwrap());
    let error = repository
        .rename(morning.id, "Evening".to_owned(), Utc::now())
        .await
        .expect_err("duplicate");
    assert!(matches!(error, RenameCategoryError::Duplicate));
    // Renaming to its own name is not a duplicate of another row.
    assert!(repository
        .rename(morning.id, "Dawn".to_owned(), Utc::now())
        .await
        .unwrap());
}

#[tokio::test]
async fn delete_reports_whether_a_row_was_removed() {
    let repository = SeaOrmCategoryRepository::new(open().await);
    let created = repository
        .insert("Morning".to_owned(), Utc::now())
        .await
        .unwrap();

    assert!(repository.delete(created.id).await.unwrap());
    assert!(!repository.delete(created.id).await.unwrap());
    assert!(repository.find_by_id(created.id).await.unwrap().is_none());
}
