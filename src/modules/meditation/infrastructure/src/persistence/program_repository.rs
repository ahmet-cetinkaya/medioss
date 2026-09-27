//! `ProgramRepository` over SeaORM/SQLite.
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use meditation_application::ports::{Program, ProgramRepository};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::persistence::entities::programs as table;

pub struct SeaOrmProgramRepository {
    db: DatabaseConnection,
}

impl SeaOrmProgramRepository {
    /// Wraps a shared connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn to_domain(model: table::Model) -> Result<Program, String> {
    let created_at = DateTime::parse_from_rfc3339(&model.created_at)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| format!("corrupt created_at {error}"))?;
    Ok(Program {
        id: model.id,
        title: model.title,
        description: model.description,
        cover_storage_key: model.cover_storage_key,
        category_id: model.category_id,
        created_at,
    })
}

#[async_trait]
impl ProgramRepository for SeaOrmProgramRepository {
    async fn insert(
        &self,
        title: String,
        description: Option<String>,
        cover_storage_key: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Result<Program, String> {
        let row = table::ActiveModel {
            title: Set(title),
            description: Set(description),
            cover_storage_key: Set(cover_storage_key),
            category_id: Set(None),
            created_at: Set(created_at.to_rfc3339()),
            ..Default::default()
        };
        let model = row.insert(&self.db).await.map_err(|e| e.to_string())?;
        to_domain(model)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Program>, String> {
        table::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .map(to_domain)
            .transpose()
    }

    async fn list_all(&self) -> Result<Vec<Program>, String> {
        table::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(to_domain)
            .collect()
    }

    async fn update(
        &self,
        id: i64,
        title: String,
        description: Option<String>,
    ) -> Result<bool, String> {
        let Some(model) = table::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
        else {
            return Ok(false);
        };
        let mut row: table::ActiveModel = model.into();
        row.title = Set(title);
        row.description = Set(description);
        row.update(&self.db).await.map_err(|e| e.to_string())?;
        Ok(true)
    }

    async fn delete(&self, id: i64) -> Result<Option<Program>, String> {
        let Some(model) = table::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
        else {
            return Ok(None);
        };
        let program = to_domain(model.clone())?;
        table::Entity::delete_by_id(model.id)
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Some(program))
    }

    async fn set_category(&self, id: i64, category_id: Option<i64>) -> Result<bool, String> {
        let Some(model) = table::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
        else {
            return Ok(false);
        };
        let mut row: table::ActiveModel = model.into();
        row.category_id = Set(category_id);
        row.update(&self.db).await.map_err(|e| e.to_string())?;
        Ok(true)
    }

    async fn clear_category(&self, category_id: i64) -> Result<u64, String> {
        // One statement instead of N row loads: `UPDATE ... SET category_id =
        // NULL WHERE category_id = ?` via the query builder.
        let result = table::Entity::update_many()
            .col_expr(
                table::Column::CategoryId,
                sea_orm::sea_query::Expr::value(Option::<i64>::None),
            )
            .filter(table::Column::CategoryId.eq(category_id))
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(result.rows_affected)
    }
}
