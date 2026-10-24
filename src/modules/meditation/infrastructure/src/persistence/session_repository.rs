//! `SessionRepository` over SeaORM/SQLite: the UNIQUE (user, meditation,
//! started_at) triple is the completion-report idempotency key.
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::UserId;
use meditation_application::ports::{MeditationSession, SessionRepository};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};

use crate::persistence::entities::sessions as table;

pub struct SeaOrmSessionRepository {
    db: DatabaseConnection,
}

impl SeaOrmSessionRepository {
    /// Wraps a shared connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn to_domain(model: table::Model) -> Result<MeditationSession, String> {
    let started_at = DateTime::parse_from_rfc3339(&model.started_at)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| format!("corrupt started_at {error}"))?;
    let finished_at = DateTime::parse_from_rfc3339(&model.finished_at)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| format!("corrupt finished_at {error}"))?;
    Ok(MeditationSession {
        id: model.id,
        user_id: UserId::new(model.user_id),
        meditation_id: model.meditation_id,
        started_at,
        finished_at,
    })
}

#[async_trait]
impl SessionRepository for SeaOrmSessionRepository {
    async fn record_if_new(
        &self,
        user_id: UserId,
        meditation_id: i64,
        started_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
    ) -> Result<MeditationSession, String> {
        let existing = table::Entity::find()
            .filter(table::Column::UserId.eq(user_id.value()))
            .filter(table::Column::MeditationId.eq(meditation_id))
            .filter(table::Column::StartedAt.eq(started_at.to_rfc3339()))
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        if let Some(model) = existing {
            return to_domain(model);
        }
        let row = table::ActiveModel {
            user_id: Set(user_id.value()),
            meditation_id: Set(meditation_id),
            started_at: Set(started_at.to_rfc3339()),
            finished_at: Set(finished_at.to_rfc3339()),
            ..Default::default()
        };
        match row.insert(&self.db).await {
            Ok(model) => to_domain(model),
            // A concurrent completion report inserted the same triple first:
            // idempotency means returning that row, not failing.
            Err(_) => {
                let model = table::Entity::find()
                    .filter(table::Column::UserId.eq(user_id.value()))
                    .filter(table::Column::MeditationId.eq(meditation_id))
                    .filter(table::Column::StartedAt.eq(started_at.to_rfc3339()))
                    .one(&self.db)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or("session insert conflict lost the race")?;
                to_domain(model)
            }
        }
    }

    async fn count_for(&self, user_id: UserId, meditation_id: i64) -> Result<i64, String> {
        table::Entity::find()
            .filter(table::Column::UserId.eq(user_id.value()))
            .filter(table::Column::MeditationId.eq(meditation_id))
            .count(&self.db)
            .await
            .map_err(|e| e.to_string())
            .map(|count| count as i64)
    }

    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<MeditationSession>, String> {
        table::Entity::find()
            .filter(table::Column::UserId.eq(user_id.value()))
            .all(&self.db)
            .await
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(to_domain)
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
#[path = "session_repository.tests.rs"]
mod tests;
