//! `FavoriteRepository` over SeaORM/SQLite, user-scoped.
use std::collections::HashSet;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::UserId;
use meditation_application::ports::FavoriteRepository;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};

use crate::persistence::entities::favorites as table;

pub struct SeaOrmFavoriteRepository {
    db: DatabaseConnection,
}

impl SeaOrmFavoriteRepository {
    /// Wraps a shared connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl FavoriteRepository for SeaOrmFavoriteRepository {
    async fn add(
        &self,
        user_id: UserId,
        meditation_id: i64,
        created_at: DateTime<Utc>,
    ) -> Result<(), String> {
        let row = table::ActiveModel {
            user_id: Set(user_id.value()),
            meditation_id: Set(meditation_id),
            created_at: Set(created_at.to_rfc3339()),
        };
        // A repeat add keeps the original created_at, so re-favoriting does
        // not reorder the list.
        table::Entity::insert(row)
            .on_conflict(
                OnConflict::columns([table::Column::UserId, table::Column::MeditationId])
                    .do_nothing()
                    .to_owned(),
            )
            .do_nothing()
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn remove(&self, user_id: UserId, meditation_id: i64) -> Result<bool, String> {
        let removed = table::Entity::delete_by_id((user_id.value(), meditation_id))
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected;
        Ok(removed > 0)
    }

    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<i64>, String> {
        table::Entity::find()
            .select_only()
            .column(table::Column::MeditationId)
            .filter(table::Column::UserId.eq(user_id.value()))
            .order_by_desc(table::Column::CreatedAt)
            .order_by_desc(table::Column::MeditationId)
            .into_tuple::<i64>()
            .all(&self.db)
            .await
            .map_err(|e| e.to_string())
    }

    async fn ids_for_user(&self, user_id: UserId) -> Result<HashSet<i64>, String> {
        Ok(self.list_for_user(user_id).await?.into_iter().collect())
    }

    async fn delete_for_meditation(&self, meditation_id: i64) -> Result<u64, String> {
        table::Entity::delete_many()
            .filter(table::Column::MeditationId.eq(meditation_id))
            .exec(&self.db)
            .await
            .map(|result| result.rows_affected)
            .map_err(|e| e.to_string())
    }

    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, String> {
        table::Entity::delete_many()
            .filter(table::Column::UserId.eq(user_id.value()))
            .exec(&self.db)
            .await
            .map(|result| result.rows_affected)
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
#[path = "favorite_repository.tests.rs"]
mod tests;
