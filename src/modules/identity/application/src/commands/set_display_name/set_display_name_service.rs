use std::sync::Arc;

use super::set_display_name::SetDisplayName;
use super::set_display_name_handler::SetDisplayNameHandler;
use crate::ports::UserRepository;
use crate::ApplicationError;
use identity_domain::User;
use kernel::UserId;

pub struct SetDisplayNameService {
    repository: Arc<dyn UserRepository>,
}

impl SetDisplayNameService {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    async fn set_display_name(
        &self,
        user_id: UserId,
        display_name: Option<identity_domain::DisplayName>,
    ) -> Result<User, ApplicationError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await
            .map_err(ApplicationError::from)?
            .ok_or(ApplicationError::UserNotFound)?;

        self.repository
            .set_profile(user_id, display_name, user.avatar())
            .await
            .map_err(ApplicationError::from)
    }
}

#[async_trait::async_trait]
impl SetDisplayNameHandler for SetDisplayNameService {
    async fn handle(&self, command: SetDisplayName) -> Result<User, ApplicationError> {
        self.set_display_name(command.user_id, command.display_name)
            .await
    }
}

#[cfg(test)]
#[path = "set_display_name_service.tests.rs"]
mod tests;
