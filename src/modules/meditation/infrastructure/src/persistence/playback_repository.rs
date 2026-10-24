//! `PlaybackStateRepository` over SeaORM/SQLite.
use async_trait::async_trait;
use kernel::UserId;
use meditation_application::ports::{PlaybackState, PlaybackStateRepository};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

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

    async fn list_touched(
        &self,
        user_id: UserId,
    ) -> Result<Vec<(i64, chrono::DateTime<chrono::Utc>)>, String> {
        let rows = table::Entity::find()
            .filter(table::Column::UserId.eq(user_id.value()))
            .order_by_desc(table::Column::UpdatedAt)
            .all(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        rows.into_iter()
            .map(|row| {
                let updated_at = chrono::DateTime::parse_from_rfc3339(&row.updated_at)
                    .map(|value| value.with_timezone(&chrono::Utc))
                    .map_err(|error| format!("corrupt updated_at {error}"))?;
                Ok((row.meditation_id, updated_at))
            })
            .collect()
    }

    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, String> {
        let rows = table::Entity::delete_many()
            .filter(table::Column::UserId.eq(user_id.value()))
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected;
        Ok(rows)
    }
}

#[cfg(test)]
#[path = "playback_repository.tests.rs"]
mod tests;
