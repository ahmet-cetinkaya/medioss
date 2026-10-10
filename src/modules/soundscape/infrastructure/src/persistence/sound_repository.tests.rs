use std::str::FromStr;
use std::time::Duration;

use chrono::Utc;
use sea_orm::{ConnectionTrait, DatabaseConnection, SqlxSqliteConnector};
use soundscape_application::ports::{MixRepository, NewMix, NewSound, SoundRepository};
use soundscape_domain::{MixItem, STORED_CONTENT_TYPE};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};

use super::{SeaOrmMixRepository, SeaOrmSoundRepository};
use crate::migrations::migrate;
use kernel::UserId;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(5);

struct TestDatabase {
    _directory: tempfile::TempDir,
    connection: DatabaseConnection,
}

impl TestDatabase {
    async fn open() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("soundscape.sqlite");
        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
            .unwrap()
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true)
            .busy_timeout(STARTUP_TIMEOUT);
        let pool: SqlitePool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .unwrap();
        let connection = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool.clone());
        migrate(&connection).await.unwrap();
        Self {
            _directory: directory,
            connection,
        }
    }
}

fn sound(name: &str, storage_key: &str) -> NewSound {
    NewSound {
        name: name.to_owned(),
        category_id: 1,
        content_type: STORED_CONTENT_TYPE.to_owned(),
        size_bytes: 1024,
        storage_key: storage_key.to_owned(),
        duration_ms: 5_000,
        created_at: Utc::now(),
        icon_key: None,
    }
}

#[tokio::test]
async fn deleting_a_sound_compacts_remaining_mix_positions() {
    let database = TestDatabase::open().await;
    let sounds = SeaOrmSoundRepository::new(database.connection.clone());
    let mixes = SeaOrmMixRepository::new(database.connection.clone());
    let now = Utc::now();

    let kept = sounds
        .create(sound("Rain", "soundscape/rain.opus"))
        .await
        .unwrap();
    let removed = sounds
        .create(sound("Wind", "soundscape/wind.opus"))
        .await
        .unwrap();
    mixes
        .create(NewMix {
            user_id: UserId::new(7),
            name: "Duo".into(),
            items: vec![
                MixItem {
                    sound_id: removed.id,
                    volume: 70,
                    position: 0,
                },
                MixItem {
                    sound_id: kept.id,
                    volume: 40,
                    position: 1,
                },
            ],
            now,
        })
        .await
        .unwrap();

    sounds
        .delete_with_mix_items(removed.id)
        .await
        .unwrap()
        .unwrap();

    let mix = mixes.list_for(UserId::new(7)).await.unwrap();
    assert_eq!(mix.len(), 1);
    assert_eq!(
        mix[0].items,
        vec![MixItem {
            sound_id: kept.id,
            volume: 40,
            position: 0
        }],
        "the remaining item must keep a contiguous position so later reads pass domain validation"
    );
}

#[tokio::test]
async fn deleting_a_sound_from_many_mixes_compacts_each_independently() {
    let database = TestDatabase::open().await;
    let sounds = SeaOrmSoundRepository::new(database.connection.clone());
    let mixes = SeaOrmMixRepository::new(database.connection.clone());
    let now = Utc::now();

    let first = sounds
        .create(sound("Rain", "soundscape/rain.opus"))
        .await
        .unwrap();
    let middle = sounds
        .create(sound("Wind", "soundscape/wind.opus"))
        .await
        .unwrap();
    let last = sounds
        .create(sound("Waves", "soundscape/waves.opus"))
        .await
        .unwrap();
    for (index, user) in [UserId::new(7), UserId::new(8)].into_iter().enumerate() {
        mixes
            .create(NewMix {
                user_id: user,
                name: format!("Mix {index}"),
                items: vec![
                    MixItem {
                        sound_id: first.id,
                        volume: 50,
                        position: 0,
                    },
                    MixItem {
                        sound_id: middle.id,
                        volume: 50,
                        position: 1,
                    },
                    MixItem {
                        sound_id: last.id,
                        volume: 50,
                        position: 2,
                    },
                ],
                now,
            })
            .await
            .unwrap();
    }

    sounds
        .delete_with_mix_items(middle.id)
        .await
        .unwrap()
        .unwrap();

    for user in [UserId::new(7), UserId::new(8)] {
        let mix = mixes.list_for(user).await.unwrap();
        assert_eq!(mix.len(), 1);
        let ids: Vec<i64> = mix[0].items.iter().map(|item| item.sound_id).collect();
        assert_eq!(ids, vec![first.id, last.id]);
        let positions: Vec<i64> = mix[0].items.iter().map(|item| item.position).collect();
        assert_eq!(positions, vec![0, 1]);
    }
}

#[tokio::test]
async fn mixes_are_not_limited_to_three_sounds() {
    let database = TestDatabase::open().await;
    let sounds = SeaOrmSoundRepository::new(database.connection.clone());
    let mixes = SeaOrmMixRepository::new(database.connection.clone());

    let mut items = Vec::new();
    for (index, name) in ["Rain", "Wind", "Waves", "Birds", "Fire"]
        .iter()
        .enumerate()
    {
        let sound = sounds
            .create(sound(name, &format!("soundscape/{name}.opus")))
            .await
            .unwrap();
        items.push(MixItem {
            sound_id: sound.id,
            volume: 50,
            position: index as i64,
        });
    }

    mixes
        .create(NewMix {
            user_id: UserId::new(7),
            name: "Five".into(),
            items,
            now: Utc::now(),
        })
        .await
        .unwrap();

    let mix = mixes.list_for(UserId::new(7)).await.unwrap();
    assert_eq!(
        mix[0].items.len(),
        5,
        "the schema must not cap a mix at three sounds"
    );
}

#[tokio::test]
async fn deleting_a_builtin_sound_records_a_tombstone() {
    let database = TestDatabase::open().await;
    let sounds = SeaOrmSoundRepository::new(database.connection.clone());

    let builtin = sounds
        .create(sound("Rain", "soundscape/rain.opus"))
        .await
        .unwrap();
    database
        .connection
        .execute(sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "UPDATE soundscape_sounds SET builtin_id = ? WHERE id = ?",
            vec!["rain:rain".into(), builtin.id.into()],
        ))
        .await
        .unwrap();
    let uploaded = sounds
        .create(sound("Wind", "soundscape/wind.opus"))
        .await
        .unwrap();

    sounds
        .delete_with_mix_items(builtin.id)
        .await
        .unwrap()
        .unwrap();
    sounds
        .delete_with_mix_items(uploaded.id)
        .await
        .unwrap()
        .unwrap();

    let tombstones = database
        .connection
        .query_all(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT builtin_id FROM soundscape_builtin_tombstones".to_owned(),
        ))
        .await
        .unwrap()
        .iter()
        .map(|row| row.try_get::<String>("", "builtin_id").unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        tombstones,
        vec!["rain:rain".to_owned()],
        "only the built-in delete is tombstoned"
    );
}

#[tokio::test]
async fn icon_key_and_uploaded_icon_roundtrip() {
    let database = TestDatabase::open().await;
    let sounds = SeaOrmSoundRepository::new(database.connection.clone());
    let created = sounds
        .create(NewSound {
            icon_key: Some("rain".into()),
            ..sound("Rain", "soundscape/rain.opus")
        })
        .await
        .unwrap();
    assert_eq!(created.icon_key.as_deref(), Some("rain"));
    assert!(created.icon.is_none());

    let with_icon = sounds
        .set_icon(
            created.id,
            Some(soundscape_domain::SoundIcon {
                storage_key: "soundscape/icon.png".into(),
                content_type: "image/png".into(),
            }),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(with_icon.icon.unwrap().content_type, "image/png");

    let edited = sounds
        .update(created.id, "Rain", 1, None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(edited.icon_key, None);
    assert!(edited.icon.is_some(), "edit leaves the uploaded image");

    let cleared = sounds.set_icon(created.id, None).await.unwrap().unwrap();
    assert!(cleared.icon.is_none());
    assert!(sounds.set_icon(999, None).await.unwrap().is_none());
}

#[tokio::test]
async fn concurrent_creates_return_their_own_rows() {
    let database = TestDatabase::open().await;
    let sounds = SeaOrmSoundRepository::new(database.connection.clone());

    // Two overlapping admin uploads. `create` must return each request's own
    // row — a global "highest id" lookup could hand one request the other's
    // sound, which the icon attach/delete rollback path would then mutate.
    let (a, b) = tokio::join!(
        sounds.create(sound("Rain", "soundscape/rain.opus")),
        sounds.create(sound("Wind", "soundscape/wind.opus")),
    );
    let (a, b) = (a.unwrap(), b.unwrap());
    assert_ne!(a.id, b.id);
    assert_eq!(a.name, "Rain");
    assert_eq!(b.name, "Wind");
    assert_eq!(a.storage_key, "soundscape/rain.opus");
    assert_eq!(b.storage_key, "soundscape/wind.opus");
}
