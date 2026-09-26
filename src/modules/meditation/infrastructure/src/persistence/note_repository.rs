//! `NoteRepository` over SeaORM/SQLite, author-scoped.
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::UserId;
use meditation_application::ports::{Note, NoteRepository};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

use crate::persistence::entities::notes as table;

pub struct SeaOrmNoteRepository {
    db: DatabaseConnection,
}

impl SeaOrmNoteRepository {
    /// Wraps a shared connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn to_domain(model: table::Model) -> Result<Note, String> {
    let created_at = DateTime::parse_from_rfc3339(&model.created_at)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| format!("corrupt created_at {error}"))?;
    let updated_at = DateTime::parse_from_rfc3339(&model.updated_at)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| format!("corrupt updated_at {error}"))?;
    Ok(Note {
        id: model.id,
        user_id: UserId::new(model.user_id),
        meditation_id: model.meditation_id,
        text: model.text,
        created_at,
        updated_at,
    })
}

#[async_trait]
impl NoteRepository for SeaOrmNoteRepository {
    async fn insert(
        &self,
        user_id: UserId,
        meditation_id: i64,
        text: String,
        created_at: DateTime<Utc>,
    ) -> Result<Note, String> {
        let stamp = created_at.to_rfc3339();
        let row = table::ActiveModel {
            user_id: Set(user_id.value()),
            meditation_id: Set(meditation_id),
            text: Set(text),
            created_at: Set(stamp.clone()),
            updated_at: Set(stamp),
            ..Default::default()
        };
        let model = row.insert(&self.db).await.map_err(|e| e.to_string())?;
        to_domain(model)
    }

    async fn list_for(&self, user_id: UserId, meditation_id: i64) -> Result<Vec<Note>, String> {
        table::Entity::find()
            .filter(table::Column::UserId.eq(user_id.value()))
            .filter(table::Column::MeditationId.eq(meditation_id))
            // Newest first: highest id first (AUTOINCREMENT monotonic).
            .order_by_desc(table::Column::Id)
            .all(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(to_domain)
            .collect()
    }

    async fn update(
        &self,
        note_id: i64,
        user_id: UserId,
        text: String,
        updated_at: DateTime<Utc>,
    ) -> Result<bool, String> {
        // Author scoping lives in the WHERE clause: another author's note is
        // indistinguishable from a nonexistent one.
        let Some(model) = table::Entity::find_by_id(note_id)
            .filter(table::Column::UserId.eq(user_id.value()))
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
        else {
            return Ok(false);
        };
        let mut row: table::ActiveModel = model.into();
        row.text = Set(text);
        row.updated_at = Set(updated_at.to_rfc3339());
        row.update(&self.db).await.map_err(|e| e.to_string())?;
        Ok(true)
    }

    async fn delete(&self, note_id: i64, user_id: UserId) -> Result<bool, String> {
        let rows = table::Entity::delete_many()
            .filter(table::Column::Id.eq(note_id))
            .filter(table::Column::UserId.eq(user_id.value()))
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected;
        Ok(rows > 0)
    }

    async fn delete_for_meditation(&self, meditation_id: i64) -> Result<u64, String> {
        let rows = table::Entity::delete_many()
            .filter(table::Column::MeditationId.eq(meditation_id))
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected;
        Ok(rows)
    }
}
