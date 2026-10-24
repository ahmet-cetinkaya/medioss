use std::sync::Arc;

use super::list_categories_query::ListCategoriesQuery;
use super::list_categories_query_validator::ListCategoriesQueryValidator;
use crate::ports::Category;
use crate::ports::CategoryRepository;
use crate::ports::QueryHandler;
use crate::ApplicationError;

pub struct ListCategoriesQueryHandler {
    categories: Arc<dyn CategoryRepository>,
    validator: ListCategoriesQueryValidator,
}

impl ListCategoriesQueryHandler {
    pub fn new(categories: Arc<dyn CategoryRepository>) -> Self {
        Self {
            categories,
            validator: ListCategoriesQueryValidator,
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<ListCategoriesQuery> for ListCategoriesQueryHandler {
    type Output = Vec<Category>;

    async fn handle(&self, query: ListCategoriesQuery) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&query)?;
        // Lists every category (spec FR-030 views).
        self.categories
            .list_all()
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "list_categories_query_handler.tests.rs"]
mod tests;
