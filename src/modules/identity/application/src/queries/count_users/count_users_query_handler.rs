use std::sync::Arc;

use super::count_users_query::CountUsersQuery;
use super::count_users_query_validator::CountUsersQueryValidator;
use crate::ports::{QueryHandler, UserRepository};
use crate::ApplicationError;

pub struct CountUsersQueryHandler {
    repository: Arc<dyn UserRepository>,
    validator: CountUsersQueryValidator,
}

impl CountUsersQueryHandler {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self {
            repository,
            validator: CountUsersQueryValidator,
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<CountUsersQuery> for CountUsersQueryHandler {
    type Output = u64;

    async fn handle(&self, query: CountUsersQuery) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&query)?;
        self.repository
            .count()
            .await
            .map_err(ApplicationError::from)
    }
}

#[cfg(test)]
#[path = "count_users_query_handler.tests.rs"]
mod tests;
