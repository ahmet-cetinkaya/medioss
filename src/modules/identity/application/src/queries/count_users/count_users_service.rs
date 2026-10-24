use std::sync::Arc;

use super::count_users::CountUsers;
use super::count_users_handler::CountUsersHandler;
use crate::ports::UserRepository;
use crate::ApplicationError;

pub struct CountUsersService {
    repository: Arc<dyn UserRepository>,
}

impl CountUsersService {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait::async_trait]
impl CountUsersHandler for CountUsersService {
    async fn handle(&self, _query: CountUsers) -> Result<u64, ApplicationError> {
        self.repository
            .count()
            .await
            .map_err(ApplicationError::from)
    }
}

#[cfg(test)]
#[path = "count_users_service.tests.rs"]
mod tests;
