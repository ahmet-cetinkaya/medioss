use std::str::FromStr;

use chrono::Utc;
use sea_orm::{DatabaseConnection, SqlxSqliteConnector};
use soundscape_application::ports::{
    CategoryDelete, CategoryRepository, NewSound, SoundRepository,
};
use soundscape_domain::STORED_CONTENT_TYPE;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use super::{slug, SeaOrmCategoryRepository};
use crate::migrations::migrate;
use crate::persistence::SeaOrmSoundRepository;

async fn open() -> (tempfile::TempDir, DatabaseConnection) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("soundscape.sqlite");
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
        .unwrap()
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();
    let connection = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
    migrate(&connection).await.unwrap();
    (directory, connection)
}

#[test]
fn slug_lowercases_and_collapses_separators() {
    assert_eq!(slug("  Heavy   Rain!! "), "heavy-rain");
}

#[test]
fn slug_transliterates_turkish_letters() {
    assert_eq!(slug("Çağrı Işık Öğüş"), "cagri-isik-ogus");
}

#[test]
fn slug_of_nothing_alphanumeric_falls_back_to_category() {
    assert_eq!(slug("   "), "category");
    assert_eq!(slug("!!!"), "category");
}

#[tokio::test]
async fn create_trims_the_name_and_derives_the_slug() {
    let (_dir, db) = open().await;
    let repo = SeaOrmCategoryRepository::new(db);

    let created = repo.create("  Ocean Waves ").await.unwrap();

    assert_eq!(created.name, "Ocean Waves");
    assert_eq!(created.slug, "ocean-waves");
    assert_eq!(repo.find(created.id).await.unwrap(), Some(created));
}

#[tokio::test]
async fn create_rejects_a_duplicate_slug() {
    let (_dir, db) = open().await;
    let repo = SeaOrmCategoryRepository::new(db);
    repo.create("Rain").await.unwrap();

    assert!(repo.create("rain").await.is_err());
}

#[tokio::test]
async fn list_is_ordered_by_name_ignoring_case() {
    let (_dir, db) = open().await;
    let repo = SeaOrmCategoryRepository::new(db);
    repo.create("zebra").await.unwrap();
    repo.create("Apple").await.unwrap();
    repo.create("mango").await.unwrap();

    let names: Vec<String> = repo
        .list()
        .await
        .unwrap()
        .into_iter()
        .map(|c| c.name)
        .collect();

    let added: Vec<&str> = names
        .iter()
        .map(String::as_str)
        .filter(|name| ["Apple", "mango", "zebra"].contains(name))
        .collect();
    assert_eq!(added, ["Apple", "mango", "zebra"]);
    assert!(names
        .windows(2)
        .all(|pair| pair[0].to_lowercase() <= pair[1].to_lowercase()));
}

#[tokio::test]
async fn find_returns_none_for_an_unknown_id() {
    let (_dir, db) = open().await;
    let repo = SeaOrmCategoryRepository::new(db);

    assert_eq!(repo.find(999).await.unwrap(), None);
}

#[tokio::test]
async fn rename_updates_name_and_slug_or_reports_none() {
    let (_dir, db) = open().await;
    let repo = SeaOrmCategoryRepository::new(db);
    let created = repo.create("Rain").await.unwrap();

    let renamed = repo
        .rename(created.id, " Night Sky ")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(renamed.id, created.id);
    assert_eq!(renamed.name, "Night Sky");
    assert_eq!(renamed.slug, "night-sky");
    assert_eq!(repo.rename(999, "Nope").await.unwrap(), None);
}

#[tokio::test]
async fn delete_empty_distinguishes_deleted_missing_and_in_use() {
    let (_dir, db) = open().await;
    let categories = SeaOrmCategoryRepository::new(db.clone());
    let sounds = SeaOrmSoundRepository::new(db);
    let empty = categories.create("Empty").await.unwrap();
    let used = categories.create("Used").await.unwrap();
    sounds
        .create(NewSound {
            name: "Rain".into(),
            category_id: used.id,
            content_type: STORED_CONTENT_TYPE.to_owned(),
            size_bytes: 1,
            storage_key: "soundscape/rain.opus".into(),
            duration_ms: 1000,
            created_at: Utc::now(),
            icon_key: None,
        })
        .await
        .unwrap();

    assert_eq!(
        categories.delete_empty(empty.id).await.unwrap(),
        CategoryDelete::Deleted
    );
    assert_eq!(categories.find(empty.id).await.unwrap(), None);
    assert_eq!(
        categories.delete_empty(empty.id).await.unwrap(),
        CategoryDelete::NotFound
    );
    assert_eq!(
        categories.delete_empty(used.id).await.unwrap(),
        CategoryDelete::InUse
    );
    assert!(categories.find(used.id).await.unwrap().is_some());
}
