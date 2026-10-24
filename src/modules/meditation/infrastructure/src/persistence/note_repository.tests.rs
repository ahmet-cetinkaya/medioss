use chrono::{Duration, Utc};
use kernel::UserId;
use meditation_application::ports::NoteRepository;

use super::SeaOrmNoteRepository;
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
async fn insert_sets_both_timestamps_and_ownership() {
    let repository = SeaOrmNoteRepository::new(open().await);
    let created_at = Utc::now();

    let note = repository
        .insert(UserId::new(ALICE), 10, "Calm".to_owned(), created_at)
        .await
        .unwrap();

    assert_eq!(note.user_id, UserId::new(ALICE));
    assert_eq!(note.meditation_id, 10);
    assert_eq!(note.text, "Calm");
    assert_eq!(note.created_at, note.updated_at);
    assert_eq!(note.created_at.timestamp(), created_at.timestamp());
}

#[tokio::test]
async fn list_for_is_scoped_to_user_and_meditation_newest_first() {
    let repository = SeaOrmNoteRepository::new(open().await);
    let now = Utc::now();
    repository
        .insert(UserId::new(ALICE), 10, "one".to_owned(), now)
        .await
        .unwrap();
    repository
        .insert(UserId::new(ALICE), 10, "two".to_owned(), now)
        .await
        .unwrap();
    repository
        .insert(UserId::new(ALICE), 11, "other".to_owned(), now)
        .await
        .unwrap();
    repository
        .insert(UserId::new(BOB), 10, "bob".to_owned(), now)
        .await
        .unwrap();

    let texts: Vec<String> = repository
        .list_for(UserId::new(ALICE), 10)
        .await
        .unwrap()
        .into_iter()
        .map(|note| note.text)
        .collect();

    assert_eq!(texts, vec!["two", "one"]);
    assert!(repository
        .list_for(UserId::new(3), 10)
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn list_for_user_orders_by_created_at_then_id_descending() {
    let repository = SeaOrmNoteRepository::new(open().await);
    let now = Utc::now();
    repository
        .insert(
            UserId::new(ALICE),
            1,
            "old".to_owned(),
            now - Duration::seconds(60),
        )
        .await
        .unwrap();
    repository
        .insert(UserId::new(ALICE), 2, "tie-a".to_owned(), now)
        .await
        .unwrap();
    repository
        .insert(UserId::new(ALICE), 3, "tie-b".to_owned(), now)
        .await
        .unwrap();
    repository
        .insert(UserId::new(BOB), 1, "bob".to_owned(), now)
        .await
        .unwrap();

    let texts: Vec<String> = repository
        .list_for_user(UserId::new(ALICE))
        .await
        .unwrap()
        .into_iter()
        .map(|note| note.text)
        .collect();

    assert_eq!(texts, vec!["tie-b", "tie-a", "old"]);
}

#[tokio::test]
async fn update_changes_text_and_updated_at_only_for_the_author_and_meditation() {
    let repository = SeaOrmNoteRepository::new(open().await);
    let created_at = Utc::now();
    let later = created_at + Duration::seconds(30);
    let note = repository
        .insert(UserId::new(ALICE), 10, "draft".to_owned(), created_at)
        .await
        .unwrap();

    assert!(!repository
        .update(note.id, 10, UserId::new(BOB), "hijack".to_owned(), later)
        .await
        .unwrap());
    assert!(!repository
        .update(note.id, 11, UserId::new(ALICE), "wrong".to_owned(), later)
        .await
        .unwrap());
    assert!(!repository
        .update(999, 10, UserId::new(ALICE), "none".to_owned(), later)
        .await
        .unwrap());
    assert!(repository
        .update(note.id, 10, UserId::new(ALICE), "final".to_owned(), later)
        .await
        .unwrap());

    let stored = repository
        .list_for(UserId::new(ALICE), 10)
        .await
        .unwrap()
        .remove(0);
    assert_eq!(stored.text, "final");
    assert_eq!(stored.updated_at.timestamp(), later.timestamp());
    assert_eq!(stored.created_at.timestamp(), created_at.timestamp());
}

#[tokio::test]
async fn delete_is_author_and_meditation_scoped() {
    let repository = SeaOrmNoteRepository::new(open().await);
    let note = repository
        .insert(UserId::new(ALICE), 10, "x".to_owned(), Utc::now())
        .await
        .unwrap();

    assert!(!repository
        .delete(note.id, 10, UserId::new(BOB))
        .await
        .unwrap());
    assert!(!repository
        .delete(note.id, 11, UserId::new(ALICE))
        .await
        .unwrap());
    assert!(repository
        .delete(note.id, 10, UserId::new(ALICE))
        .await
        .unwrap());
    assert!(!repository
        .delete(note.id, 10, UserId::new(ALICE))
        .await
        .unwrap());
}

#[tokio::test]
async fn bulk_deletes_count_removed_rows() {
    let repository = SeaOrmNoteRepository::new(open().await);
    let now = Utc::now();
    repository
        .insert(UserId::new(ALICE), 10, "a".to_owned(), now)
        .await
        .unwrap();
    repository
        .insert(UserId::new(BOB), 10, "b".to_owned(), now)
        .await
        .unwrap();
    repository
        .insert(UserId::new(BOB), 11, "c".to_owned(), now)
        .await
        .unwrap();

    assert_eq!(repository.delete_for_meditation(10).await.unwrap(), 2);
    assert_eq!(repository.delete_for_meditation(10).await.unwrap(), 0);
    assert_eq!(
        repository
            .delete_all_for_user(UserId::new(BOB))
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        repository
            .delete_all_for_user(UserId::new(BOB))
            .await
            .unwrap(),
        0
    );
}
