use std::sync::Arc;

use super::list_users::ListUsers;
use super::list_users_handler::ListUsersHandler;
use crate::ports::UserRepository;
use crate::ApplicationError;
use identity_domain::User;

pub struct ListUsersService {
    repository: Arc<dyn UserRepository>,
}

impl ListUsersService {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait::async_trait]
impl ListUsersHandler for ListUsersService {
    async fn handle(&self, _query: ListUsers) -> Result<Vec<User>, ApplicationError> {
        self.repository.list().await.map_err(ApplicationError::from)
    }
}

#[cfg(test)]
#[path = "list_users_service.tests.rs"]
mod tests;
