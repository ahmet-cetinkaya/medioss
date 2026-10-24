use std::str::FromStr;

use chrono::{Duration, Utc};
use kernel::UserId;
use sea_orm::{DatabaseConnection, SqlxSqliteConnector};
use soundscape_application::ports::{MixRepository, NewMix, NewSound, SoundRepository};
use soundscape_domain::{MixItem, STORED_CONTENT_TYPE};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use super::SeaOrmMixRepository;
use crate::migrations::migrate;
use crate::persistence::SeaOrmSoundRepository;

const OWNER: i64 = 7;
const OTHER: i64 = 8;

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

async fn make_sound(db: &DatabaseConnection, name: &str) -> i64 {
    SeaOrmSoundRepository::new(db.clone())
        .create(NewSound {
            name: name.into(),
            category_id: 1,
            content_type: STORED_CONTENT_TYPE.to_owned(),
            size_bytes: 1,
            storage_key: format!("soundscape/{name}.opus"),
            duration_ms: 1000,
            created_at: Utc::now(),
            icon_key: None,
        })
        .await
        .unwrap()
        .id
}

fn new_mix(user: i64, name: &str, sound_ids: &[i64], now: chrono::DateTime<Utc>) -> NewMix {
    NewMix {
        user_id: UserId::new(user),
        name: name.into(),
        items: sound_ids
            .iter()
            .enumerate()
            .map(|(position, sound_id)| MixItem {
                sound_id: *sound_id,
                volume: 50,
                position: position as i64,
            })
            .collect(),
        now,
    }
}

#[tokio::test]
async fn create_then_find_roundtrips_items_in_order() {
    let (_dir, db) = open().await;
    let a = make_sound(&db, "a").await;
    let b = make_sound(&db, "b").await;
    let repo = SeaOrmMixRepository::new(db);

    let created = repo
        .create(new_mix(OWNER, "Calm", &[a, b], Utc::now()))
        .await
        .unwrap();
    let found = repo
        .find_for(created.id, UserId::new(OWNER))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(found.name, "Calm");
    assert_eq!(
        found.items.iter().map(|i| i.sound_id).collect::<Vec<_>>(),
        [a, b]
    );
    assert_eq!(found, created);
}

#[tokio::test]
async fn mixes_are_invisible_to_other_users() {
    let (_dir, db) = open().await;
    let a = make_sound(&db, "a").await;
    let repo = SeaOrmMixRepository::new(db);
    let created = repo
        .create(new_mix(OWNER, "Mine", &[a], Utc::now()))
        .await
        .unwrap();

    assert_eq!(
        repo.find_for(created.id, UserId::new(OTHER)).await.unwrap(),
        None
    );
    assert!(repo.list_for(UserId::new(OTHER)).await.unwrap().is_empty());
    assert!(!repo
        .delete_for(created.id, UserId::new(OTHER))
        .await
        .unwrap());
    assert!(repo
        .find_for(created.id, UserId::new(OWNER))
        .await
        .unwrap()
        .is_some());
}

#[tokio::test]
async fn list_for_orders_most_recently_updated_first() {
    let (_dir, db) = open().await;
    let a = make_sound(&db, "a").await;
    let repo = SeaOrmMixRepository::new(db);
    let base = Utc::now();
    repo.create(new_mix(OWNER, "old", &[a], base))
        .await
        .unwrap();
    repo.create(new_mix(OWNER, "new", &[a], base + Duration::seconds(10)))
        .await
        .unwrap();

    let names: Vec<String> = repo
        .list_for(UserId::new(OWNER))
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.name)
        .collect();

    assert_eq!(names, ["new", "old"]);
}

#[tokio::test]
async fn replace_swaps_name_and_items_for_the_owner_only() {
    let (_dir, db) = open().await;
    let a = make_sound(&db, "a").await;
    let b = make_sound(&db, "b").await;
    let repo = SeaOrmMixRepository::new(db);
    let created = repo
        .create(new_mix(OWNER, "v1", &[a], Utc::now()))
        .await
        .unwrap();

    let replaced = repo
        .replace(created.id, new_mix(OWNER, "v2", &[b], Utc::now()))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(replaced.name, "v2");
    assert_eq!(
        replaced
            .items
            .iter()
            .map(|i| i.sound_id)
            .collect::<Vec<_>>(),
        [b]
    );
    assert_eq!(
        repo.replace(created.id, new_mix(OTHER, "hijack", &[a], Utc::now()))
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        repo.find_for(created.id, UserId::new(OWNER))
            .await
            .unwrap()
            .unwrap()
            .name,
        "v2"
    );
    assert_eq!(
        repo.replace(999, new_mix(OWNER, "x", &[a], Utc::now()))
            .await
            .unwrap(),
        None
    );
}

#[tokio::test]
async fn delete_for_removes_an_owned_mix_once() {
    let (_dir, db) = open().await;
    let a = make_sound(&db, "a").await;
    let repo = SeaOrmMixRepository::new(db);
    let created = repo
        .create(new_mix(OWNER, "gone", &[a], Utc::now()))
        .await
        .unwrap();

    assert!(repo
        .delete_for(created.id, UserId::new(OWNER))
        .await
        .unwrap());
    assert!(!repo
        .delete_for(created.id, UserId::new(OWNER))
        .await
        .unwrap());
    assert_eq!(
        repo.find_for(created.id, UserId::new(OWNER)).await.unwrap(),
        None
    );
}

#[tokio::test]
async fn delete_all_for_user_counts_and_spares_other_users() {
    let (_dir, db) = open().await;
    let a = make_sound(&db, "a").await;
    let repo = SeaOrmMixRepository::new(db);
    repo.create(new_mix(OWNER, "one", &[a], Utc::now()))
        .await
        .unwrap();
    repo.create(new_mix(OWNER, "two", &[a], Utc::now()))
        .await
        .unwrap();
    repo.create(new_mix(OTHER, "keep", &[a], Utc::now()))
        .await
        .unwrap();

    assert_eq!(
        repo.delete_all_for_user(UserId::new(OWNER)).await.unwrap(),
        2
    );

    assert!(repo.list_for(UserId::new(OWNER)).await.unwrap().is_empty());
    assert_eq!(repo.list_for(UserId::new(OTHER)).await.unwrap().len(), 1);
    assert_eq!(
        repo.delete_all_for_user(UserId::new(OWNER)).await.unwrap(),
        0
    );
}
