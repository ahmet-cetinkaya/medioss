use std::sync::Arc;

use super::set_user_status::SetUserStatus;
use super::set_user_status_handler::SetUserStatusHandler;
use crate::ports::UserRepository;
use crate::ApplicationError;
use kernel::Role;

pub struct SetUserStatusService {
    repository: Arc<dyn UserRepository>,
}

impl SetUserStatusService {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait::async_trait]
impl SetUserStatusHandler for SetUserStatusService {
    async fn handle(&self, command: SetUserStatus) -> Result<(), ApplicationError> {
        if command.actor.role() != Role::Admin {
            return Err(ApplicationError::Forbidden);
        }
        if command.actor.user_id() == command.target && command.disabled {
            return Err(ApplicationError::LastActiveAdmin);
        }
        let user = self
            .repository
            .find_by_id(command.target)
            .await
            .map_err(ApplicationError::from)?
            .ok_or(ApplicationError::UserNotFound)?;
        if command.disabled && user.role() == Role::Admin {
            let active_admins = self
                .repository
                .count_active_admins()
                .await
                .map_err(ApplicationError::from)?;
            if active_admins <= 1 {
                return Err(ApplicationError::LastActiveAdmin);
            }
        }
        self.repository
            .set_disabled(command.target, command.disabled)
            .await
            .map_err(ApplicationError::from)
    }
}

#[cfg(test)]
#[path = "set_user_status_service.tests.rs"]
mod tests;
