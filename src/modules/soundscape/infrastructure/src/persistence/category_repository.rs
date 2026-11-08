use crate::persistence::slug;
use async_trait::async_trait;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use soundscape_application::ports::{CategoryDelete, CategoryRepository};
use soundscape_domain::SoundCategory;

pub struct SeaOrmCategoryRepository {
    db: DatabaseConnection,
}
impl SeaOrmCategoryRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
fn row(row: &sea_orm::QueryResult) -> Result<SoundCategory, String> {
    SoundCategory::new(
        row.try_get("", "id").map_err(|e| e.to_string())?,
        &row.try_get::<String>("", "name")
            .map_err(|e| e.to_string())?,
        &row.try_get::<String>("", "slug")
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
#[async_trait]
impl CategoryRepository for SeaOrmCategoryRepository {
    async fn list(&self) -> Result<Vec<SoundCategory>, String> {
        self.db
            .query_all(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT id,name,slug FROM soundscape_categories ORDER BY name COLLATE NOCASE,id"
                    .to_owned(),
            ))
            .await
            .map_err(|e| e.to_string())?
            .iter()
            .map(row)
            .collect()
    }
    async fn find(&self, id: i64) -> Result<Option<SoundCategory>, String> {
        let r = self
            .db
            .query_one(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT id,name,slug FROM soundscape_categories WHERE id=?",
                [id.into()],
            ))
            .await
            .map_err(|e| e.to_string())?;
        r.as_ref().map(row).transpose()
    }
    async fn create(&self, name: &str) -> Result<SoundCategory, String> {
        let slug = slug(name);
        // RETURNING binds the returned row to this INSERT; a global
        // last-inserted select could pick up a concurrent category.
        self.db
            .query_one(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                "INSERT INTO soundscape_categories(slug,name) VALUES(?,?) RETURNING id,name,slug",
                [slug.into(), name.trim().into()],
            ))
            .await
            .map_err(|e| e.to_string())?
            .as_ref()
            .map(row)
            .transpose()?
            .ok_or_else(|| "category insert missing".into())
    }
    async fn rename(&self, id: i64, name: &str) -> Result<Option<SoundCategory>, String> {
        let slug = slug(name);
        let result = self
            .db
            .execute(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                "UPDATE soundscape_categories SET slug=?,name=? WHERE id=?",
                [slug.into(), name.trim().into(), id.into()],
            ))
            .await
            .map_err(|e| e.to_string())?;
        if result.rows_affected() == 0 {
            return Ok(None);
        };
        self.find(id).await
    }
    async fn delete_empty(&self, id: i64) -> Result<CategoryDelete, String> {
        let in_use = self
            .db
            .query_one(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT 1 FROM soundscape_sounds WHERE category_id=? LIMIT 1",
                [id.into()],
            ))
            .await
            .map_err(|e| e.to_string())?
            .is_some();
        if in_use {
            return Ok(CategoryDelete::InUse);
        };
        let r = self
            .db
            .execute(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                "DELETE FROM soundscape_categories WHERE id=?",
                [id.into()],
            ))
            .await
            .map_err(|e| e.to_string())?;
        Ok(if r.rows_affected() == 0 {
            CategoryDelete::NotFound
        } else {
            CategoryDelete::Deleted
        })
    }
}

#[cfg(test)]
#[path = "category_repository.tests.rs"]
mod tests;
