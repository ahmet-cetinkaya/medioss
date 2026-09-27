//! `MeditationRepository` over SeaORM/SQLite.
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use meditation_application::ports::MeditationRepository;
use meditation_domain::Meditation;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryOrder, Set};

use crate::persistence::entities::meditations as table;

pub struct SeaOrmMeditationRepository {
    db: DatabaseConnection,
}

impl SeaOrmMeditationRepository {
    /// Wraps a shared connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn to_domain(model: table::Model) -> Result<Meditation, String> {
    let created_at = DateTime::parse_from_rfc3339(&model.created_at)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| format!("corrupt created_at {error}"))?;
    Meditation::new(
        model.id,
        &model.name,
        &model.content_type,
        model.size_bytes,
        &model.storage_key,
        model.duration_ms,
        created_at,
    )
    .map_err(|error| error.to_string())
}

#[async_trait]
impl MeditationRepository for SeaOrmMeditationRepository {
    async fn insert(
        &self,
        name: String,
        content_type: &str,
        size_bytes: i64,
        storage_key: String,
        duration_ms: i64,
        created_at: DateTime<Utc>,
    ) -> Result<Meditation, String> {
        let row = table::ActiveModel {
            name: Set(name),
            content_type: Set(content_type.to_owned()),
            size_bytes: Set(size_bytes),
            storage_key: Set(storage_key),
            duration_ms: Set(duration_ms),
            created_at: Set(created_at.to_rfc3339()),
            ..Default::default()
        };
        let model = row.insert(&self.db).await.map_err(|e| e.to_string())?;
        to_domain(model)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Meditation>, String> {
        table::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .map(to_domain)
            .transpose()
    }

    async fn list_all(&self) -> Result<Vec<Meditation>, String> {
        table::Entity::find()
            .order_by_asc(table::Column::Id)
            .all(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(to_domain)
            .collect::<Result<Vec<_>, _>>()
            // Newest first = highest id first (AUTOINCREMENT monotonic).
            .map(|mut rows| {
                rows.reverse();
                rows
            })
    }

    async fn set_storage_key(&self, id: i64, key: &str) -> Result<(), String> {
        let Some(model) = table::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
        else {
            return Err(format!("meditation {id} vanished mid-relocation"));
        };
        let mut row: table::ActiveModel = model.into();
        row.storage_key = Set(key.to_owned());
        row.update(&self.db).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> Result<bool, String> {
        let rows = table::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected;
        Ok(rows > 0)
    }
}
