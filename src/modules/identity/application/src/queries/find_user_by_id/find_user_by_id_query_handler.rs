use std::sync::Arc;

use super::find_user_by_id_query::FindUserByIdQuery;
use super::find_user_by_id_query_validator::FindUserByIdQueryValidator;
use crate::ports::{QueryHandler, UserRepository};
use crate::ApplicationError;
use identity_domain::User;

pub struct FindUserByIdQueryHandler {
    repository: Arc<dyn UserRepository>,
    validator: FindUserByIdQueryValidator,
}

impl FindUserByIdQueryHandler {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self {
            repository,
            validator: FindUserByIdQueryValidator::default(),
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<FindUserByIdQuery> for FindUserByIdQueryHandler {
    type Output = Option<User>;

    async fn handle(&self, query: FindUserByIdQuery) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&query)?;
        self.repository
            .find_by_id(query.id)
            .await
            .map_err(ApplicationError::from)
    }
}

#[cfg(test)]
#[path = "find_user_by_id_query_handler.tests.rs"]
mod tests;
