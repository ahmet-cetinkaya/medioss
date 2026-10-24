use std::sync::Arc;

use super::setup_status::IsSetupRequired;
use super::setup_status_handler::IsSetupRequiredHandler;
use crate::ports::UserRepository;
use crate::queries::count_users::{CountUsers, CountUsersHandler, CountUsersService};
use crate::ApplicationError;

pub struct SetupStatusService {
    count_users: CountUsersService,
}

impl SetupStatusService {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self {
            count_users: CountUsersService::new(repository),
        }
    }
}

#[async_trait::async_trait]
impl IsSetupRequiredHandler for SetupStatusService {
    async fn handle(&self, _query: IsSetupRequired) -> Result<bool, ApplicationError> {
        Ok(self.count_users.handle(CountUsers).await? == 0)
    }
}

#[cfg(test)]
#[path = "setup_status_service.tests.rs"]
mod tests;
