use std::sync::Arc;

use super::find_user_by_id::FindUserById;
use super::find_user_by_id_handler::FindUserByIdHandler;
use crate::ports::UserRepository;
use crate::ApplicationError;
use identity_domain::User;

pub struct FindUserByIdService {
    repository: Arc<dyn UserRepository>,
}

impl FindUserByIdService {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait::async_trait]
impl FindUserByIdHandler for FindUserByIdService {
    async fn handle(&self, query: FindUserById) -> Result<Option<User>, ApplicationError> {
        self.repository
            .find_by_id(query.id)
            .await
            .map_err(ApplicationError::from)
    }
}

#[cfg(test)]
#[path = "find_user_by_id_service.tests.rs"]
mod tests;
