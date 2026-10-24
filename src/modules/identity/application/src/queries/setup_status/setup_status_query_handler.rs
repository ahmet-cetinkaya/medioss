use std::sync::Arc;

use super::setup_status_query::SetupStatusQuery;
use super::setup_status_query_validator::SetupStatusQueryValidator;
use crate::ports::{QueryHandler, UserRepository};
use crate::queries::count_users::{CountUsersQuery, CountUsersQueryHandler};
use crate::ApplicationError;

pub struct SetupStatusQueryHandler {
    count_users: CountUsersQueryHandler,
    validator: SetupStatusQueryValidator,
}

impl SetupStatusQueryHandler {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self {
            count_users: CountUsersQueryHandler::new(repository),
            validator: SetupStatusQueryValidator::default(),
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<SetupStatusQuery> for SetupStatusQueryHandler {
    type Output = bool;

    async fn handle(&self, query: SetupStatusQuery) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&query)?;
        Ok(self.count_users.handle(CountUsersQuery).await? == 0)
    }
}

#[cfg(test)]
#[path = "setup_status_query_handler.tests.rs"]
mod tests;
