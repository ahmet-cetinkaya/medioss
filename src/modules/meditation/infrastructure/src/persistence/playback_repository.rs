//! `PlaybackStateRepository` over SeaORM/SQLite.
use async_trait::async_trait;
use kernel::UserId;
use meditation_application::ports::{PlaybackState, PlaybackStateRepository};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

use crate::persistence::entities::playback_state as table;

pub struct SeaOrmPlaybackRepository {
    db: DatabaseConnection,
}

impl SeaOrmPlaybackRepository {
    /// Wraps a shared connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn upsert(
        &self,
        user_id: UserId,
        meditation_id: i64,
        mutate: impl Fn(&mut table::ActiveModel),
    ) -> Result<(), String> {
        let existing = table::Entity::find_by_id((user_id.value(), meditation_id))
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        // Composite-key ActiveModels cannot rely on `save`/`update` to pick
        // the right statement: track existence explicitly from the find
        // above instead of inferring it from which fields are set.
        let existed = existing.is_some();
        let mut row: table::ActiveModel = match existing {
            Some(model) => model.into(),
            None => table::ActiveModel {
                user_id: Set(user_id.value()),
                meditation_id: Set(meditation_id),
                position_ms: Set(0),
                completed: Set(false),
                updated_at: Set(chrono::Utc::now().to_rfc3339()),
            },
        };
        mutate(&mut row);
        row.updated_at = Set(chrono::Utc::now().to_rfc3339());
        if !existed {
            row.insert(&self.db).await.map_err(|e| e.to_string())?;
        } else {
            row.update(&self.db).await.map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

#[async_trait]
impl PlaybackStateRepository for SeaOrmPlaybackRepository {
    async fn upsert_position(
        &self,
        user_id: UserId,
        meditation_id: i64,
        position_ms: i64,
    ) -> Result<(), String> {
        self.upsert(user_id, meditation_id, |row| {
            row.position_ms = Set(position_ms);
        })
        .await
    }

    async fn set_completed(
        &self,
        user_id: UserId,
        meditation_id: i64,
        completed: bool,
    ) -> Result<(), String> {
        self.upsert(user_id, meditation_id, |row| {
            row.completed = Set(completed);
        })
        .await
    }

    async fn reset_position(&self, user_id: UserId, meditation_id: i64) -> Result<(), String> {
        self.upsert(user_id, meditation_id, |row| {
            row.position_ms = Set(0);
        })
        .await
    }

    async fn get(
        &self,
        user_id: UserId,
        meditation_id: i64,
    ) -> Result<Option<PlaybackState>, String> {
        let model = table::Entity::find_by_id((user_id.value(), meditation_id))
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(model.map(|row| PlaybackState {
            position_ms: row.position_ms,
            completed: row.completed,
        }))
    }
}
