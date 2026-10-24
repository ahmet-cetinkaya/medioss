use std::sync::Arc;

use super::list_users_query::ListUsersQuery;
use super::list_users_query_validator::ListUsersQueryValidator;
use crate::ports::{QueryHandler, UserRepository};
use crate::ApplicationError;
use identity_domain::User;

pub struct ListUsersQueryHandler {
    repository: Arc<dyn UserRepository>,
    validator: ListUsersQueryValidator,
}

impl ListUsersQueryHandler {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self {
            repository,
            validator: ListUsersQueryValidator,
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<ListUsersQuery> for ListUsersQueryHandler {
    type Output = Vec<User>;

    async fn handle(&self, query: ListUsersQuery) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&query)?;
        self.repository.list().await.map_err(ApplicationError::from)
    }
}

#[cfg(test)]
#[path = "list_users_query_handler.tests.rs"]
mod tests;
