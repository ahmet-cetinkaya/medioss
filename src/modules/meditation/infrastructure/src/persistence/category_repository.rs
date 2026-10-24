//! `CategoryRepository` over SeaORM/SQLite.
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use meditation_application::ports::{Category, CategoryRepository, RenameCategoryError};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::persistence::entities::categories as table;

pub struct SeaOrmCategoryRepository {
    db: DatabaseConnection,
}

impl SeaOrmCategoryRepository {
    /// Wraps a shared connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn is_duplicate_name(error: &sea_orm::DbErr) -> bool {
    error
        .to_string()
        .contains("UNIQUE constraint failed: meditation_categories.name")
}

fn to_domain(model: table::Model) -> Result<Category, String> {
    let created_at = DateTime::parse_from_rfc3339(&model.created_at)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| format!("corrupt created_at {error}"))?;
    Ok(Category {
        id: model.id,
        name: model.name,
        created_at,
    })
}

#[async_trait]
impl CategoryRepository for SeaOrmCategoryRepository {
    async fn insert(&self, name: String, created_at: DateTime<Utc>) -> Result<Category, String> {
        let row = table::ActiveModel {
            name: Set(name),
            created_at: Set(created_at.to_rfc3339()),
            ..Default::default()
        };
        let model = row.insert(&self.db).await.map_err(|e| e.to_string())?;
        to_domain(model)
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Category>, String> {
        table::Entity::find()
            .filter(table::Column::Name.eq(name))
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .map(to_domain)
            .transpose()
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Category>, String> {
        table::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .map(to_domain)
            .transpose()
    }

    async fn list_all(&self) -> Result<Vec<Category>, String> {
        table::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(to_domain)
            .collect()
    }

    async fn rename(
        &self,
        id: i64,
        name: String,
        _updated_at: DateTime<Utc>,
    ) -> Result<bool, RenameCategoryError> {
        // The table stores only created_at; the timestamp parameter keeps
        // the port honest for stores that track updates.
        let row = table::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|error| RenameCategoryError::Repository(error.to_string()))?;
        let Some(model) = row else {
            return Ok(false);
        };
        let mut active: table::ActiveModel = model.into();
        active.name = Set(name);
        active.update(&self.db).await.map_err(|error| {
            if is_duplicate_name(&error) {
                RenameCategoryError::Duplicate
            } else {
                RenameCategoryError::Repository(error.to_string())
            }
        })?;
        Ok(true)
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

#[cfg(test)]
#[path = "category_repository.tests.rs"]
mod tests;
