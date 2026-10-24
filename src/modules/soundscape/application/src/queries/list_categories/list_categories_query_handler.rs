use std::sync::Arc;

use super::list_categories_query::ListCategoriesQuery;
use super::list_categories_query_validator::ListCategoriesQueryValidator;
use crate::ports::{CategoryRepository, QueryHandler};
use crate::ApplicationError;
use soundscape_domain::SoundCategory;

pub struct ListCategoriesQueryHandler {
    categories: Arc<dyn CategoryRepository>,
    validator: ListCategoriesQueryValidator,
}

impl ListCategoriesQueryHandler {
    pub fn new(categories: Arc<dyn CategoryRepository>) -> Self {
        Self {
            categories,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<ListCategoriesQuery> for ListCategoriesQueryHandler {
    type Output = Vec<SoundCategory>;

    async fn handle(
        &self,
        query: ListCategoriesQuery,
    ) -> Result<Vec<SoundCategory>, ApplicationError> {
        self.validator.validate(&query)?;
        self.categories
            .list()
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "list_categories_query_handler.tests.rs"]
mod tests;
