use std::sync::Arc;

use super::{CreateCategory, CreateCategoryHandler, CreateCategoryService};
use crate::ports::{CategoryDelete, CategoryRepository};
use crate::ApplicationError;
use async_trait::async_trait;

struct Categories;
#[async_trait]
impl CategoryRepository for Categories {
    async fn list(&self) -> Result<Vec<soundscape_domain::SoundCategory>, String> {
        Ok(Vec::new())
    }
    async fn find(&self, _: i64) -> Result<Option<soundscape_domain::SoundCategory>, String> {
        Ok(None)
    }
    async fn create(&self, _: &str) -> Result<soundscape_domain::SoundCategory, String> {
        Err("UNIQUE constraint failed: soundscape_categories.name".into())
    }
    async fn rename(
        &self,
        _: i64,
        _: &str,
    ) -> Result<Option<soundscape_domain::SoundCategory>, String> {
        Ok(None)
    }
    async fn delete_empty(&self, _: i64) -> Result<CategoryDelete, String> {
        Ok(CategoryDelete::NotFound)
    }
}

#[tokio::test]
async fn duplicate_category_is_a_conflict_shaped_application_error() {
    let service = CreateCategoryService::new(Arc::new(Categories));
    assert_eq!(
        service
            .handle(CreateCategory {
                name: "Rain".into()
            })
            .await,
        Err(ApplicationError::Duplicate)
    );
}
