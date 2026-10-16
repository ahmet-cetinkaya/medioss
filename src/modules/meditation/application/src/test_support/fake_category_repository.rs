use chrono::{DateTime, Utc};

use super::repo_state::RepoState;
use crate::ports::{Category, CategoryRepository, RenameCategoryError};

/// CategoryRepository fake.
#[derive(Clone)]
pub struct FakeCategoryRepository(pub RepoState);

#[async_trait::async_trait]
impl CategoryRepository for FakeCategoryRepository {
    async fn insert(&self, name: String, created_at: DateTime<Utc>) -> Result<Category, String> {
        let mut rows = self.0.categories.lock().unwrap();
        let id = RepoState::next_id(&rows, |category| category.id);
        let category = Category {
            id,
            name,
            created_at,
        };
        rows.push(category.clone());
        Ok(category)
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Category>, String> {
        Ok(self
            .0
            .categories
            .lock()
            .unwrap()
            .iter()
            .find(|category| category.name == name)
            .cloned())
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Category>, String> {
        Ok(self
            .0
            .categories
            .lock()
            .unwrap()
            .iter()
            .find(|category| category.id == id)
            .cloned())
    }

    async fn list_all(&self) -> Result<Vec<Category>, String> {
        Ok(self.0.categories.lock().unwrap().clone())
    }

    async fn rename(
        &self,
        id: i64,
        name: String,
        _updated_at: DateTime<Utc>,
    ) -> Result<bool, RenameCategoryError> {
        let mut rows = self.0.categories.lock().unwrap();
        if rows
            .iter()
            .any(|category| category.id != id && category.name == name)
        {
            return Err(RenameCategoryError::Duplicate);
        }
        let Some(category) = rows.iter_mut().find(|category| category.id == id) else {
            return Ok(false);
        };
        category.name = name;
        Ok(true)
    }

    async fn delete(&self, id: i64) -> Result<bool, String> {
        let mut rows = self.0.categories.lock().unwrap();
        let len = rows.len();
        rows.retain(|category| category.id != id);
        Ok(rows.len() != len)
    }
}
