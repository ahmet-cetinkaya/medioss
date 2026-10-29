use chrono::Utc;
use meditation_application::ports::ProgramRepository;

use super::SeaOrmProgramRepository;
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
async fn insert_round_trips_and_derives_a_slugged_folder() {
    let repository = SeaOrmProgramRepository::new(open().await);
    let created_at = Utc::now();

    let program = repository
        .insert(
            "Çalışma Günü!".to_owned(),
            Some("About".to_owned()),
            created_at,
        )
        .await
        .unwrap();
    let found = repository.find_by_id(program.id).await.unwrap().unwrap();

    assert_eq!(found, program);
    assert_eq!(found.title, "Çalışma Günü!");
    assert_eq!(found.description.as_deref(), Some("About"));
    assert!(found.folder.starts_with("calisma-gunu-"));
    assert_eq!(found.cover_storage_key, None);
    assert_eq!(found.category_id, None);
    assert_eq!(found.created_at.timestamp(), created_at.timestamp());
}

#[tokio::test]
async fn same_title_gets_distinct_folders() {
    let repository = SeaOrmProgramRepository::new(open().await);

    let first = repository
        .insert("Same".to_owned(), None, Utc::now())
        .await
        .unwrap();
    let second = repository
        .insert("Same".to_owned(), None, Utc::now())
        .await
        .unwrap();

    assert_ne!(first.folder, second.folder);
}

#[tokio::test]
async fn find_by_id_and_list_all_handle_empty_store() {
    let repository = SeaOrmProgramRepository::new(open().await);

    assert!(repository.find_by_id(1).await.unwrap().is_none());
    assert!(repository.list_all().await.unwrap().is_empty());
}

#[tokio::test]
async fn list_all_returns_every_program() {
    let repository = SeaOrmProgramRepository::new(open().await);
    repository
        .insert("A".to_owned(), None, Utc::now())
        .await
        .unwrap();
    repository
        .insert("B".to_owned(), None, Utc::now())
        .await
        .unwrap();

    assert_eq!(repository.list_all().await.unwrap().len(), 2);
}

#[tokio::test]
async fn update_changes_title_and_description_but_not_folder() {
    let repository = SeaOrmProgramRepository::new(open().await);
    let program = repository
        .insert("Old".to_owned(), Some("d".to_owned()), Utc::now())
        .await
        .unwrap();

    assert!(repository
        .update(program.id, "New".to_owned(), None)
        .await
        .unwrap());
    assert!(!repository.update(999, "X".to_owned(), None).await.unwrap());

    let found = repository.find_by_id(program.id).await.unwrap().unwrap();
    assert_eq!(found.title, "New");
    assert_eq!(found.description, None);
    assert_eq!(found.folder, program.folder);
}

#[tokio::test]
async fn set_cover_key_stores_key_or_reports_missing() {
    let repository = SeaOrmProgramRepository::new(open().await);
    let program = repository
        .insert("P".to_owned(), None, Utc::now())
        .await
        .unwrap();

    assert!(repository
        .set_cover_key(program.id, "covers/p")
        .await
        .unwrap());
    assert!(!repository.set_cover_key(999, "covers/x").await.unwrap());
    assert_eq!(
        repository
            .find_by_id(program.id)
            .await
            .unwrap()
            .unwrap()
            .cover_storage_key
            .as_deref(),
        Some("covers/p")
    );
}

#[tokio::test]
async fn delete_returns_the_removed_program_once() {
    let repository = SeaOrmProgramRepository::new(open().await);
    let program = repository
        .insert("P".to_owned(), None, Utc::now())
        .await
        .unwrap();

    assert_eq!(
        repository.delete(program.id).await.unwrap(),
        Some(program.clone())
    );
    assert_eq!(repository.delete(program.id).await.unwrap(), None);
    assert!(repository.find_by_id(program.id).await.unwrap().is_none());
}

#[tokio::test]
async fn set_category_assigns_and_clears() {
    let repository = SeaOrmProgramRepository::new(open().await);
    let program = repository
        .insert("P".to_owned(), None, Utc::now())
        .await
        .unwrap();

    assert!(repository.set_category(program.id, Some(4)).await.unwrap());
    assert_eq!(
        repository
            .find_by_id(program.id)
            .await
            .unwrap()
            .unwrap()
            .category_id,
        Some(4)
    );
    assert!(repository.set_category(program.id, None).await.unwrap());
    assert_eq!(
        repository
            .find_by_id(program.id)
            .await
            .unwrap()
            .unwrap()
            .category_id,
        None
    );
    assert!(!repository.set_category(999, Some(1)).await.unwrap());
}

#[tokio::test]
async fn clear_category_nulls_only_matching_programs() {
    let repository = SeaOrmProgramRepository::new(open().await);
    let first = repository
        .insert("A".to_owned(), None, Utc::now())
        .await
        .unwrap();
    let second = repository
        .insert("B".to_owned(), None, Utc::now())
        .await
        .unwrap();
    let third = repository
        .insert("C".to_owned(), None, Utc::now())
        .await
        .unwrap();
    repository.set_category(first.id, Some(4)).await.unwrap();
    repository.set_category(second.id, Some(4)).await.unwrap();
    repository.set_category(third.id, Some(5)).await.unwrap();

    assert_eq!(repository.clear_category(4).await.unwrap(), 2);
    assert_eq!(repository.clear_category(4).await.unwrap(), 0);
    assert_eq!(
        repository
            .find_by_id(first.id)
            .await
            .unwrap()
            .unwrap()
            .category_id,
        None
    );
    assert_eq!(
        repository
            .find_by_id(third.id)
            .await
            .unwrap()
            .unwrap()
            .category_id,
        Some(5)
    );
}

#[tokio::test]
async fn clear_cover_key_removes_the_key_or_reports_missing() {
    let repository = SeaOrmProgramRepository::new(open().await);
    let program = repository
        .insert("P".to_owned(), None, Utc::now())
        .await
        .unwrap();
    repository
        .set_cover_key(program.id, "covers/p")
        .await
        .unwrap();

    assert!(repository.clear_cover_key(program.id).await.unwrap());
    assert!(!repository.clear_cover_key(999).await.unwrap());
    assert_eq!(
        repository
            .find_by_id(program.id)
            .await
            .unwrap()
            .unwrap()
            .cover_storage_key,
        None
    );
}
