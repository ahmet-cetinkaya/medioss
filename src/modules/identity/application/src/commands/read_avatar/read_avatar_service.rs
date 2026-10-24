use std::sync::Arc;

use super::read_avatar::ReadAvatar;
use super::read_avatar_handler::ReadAvatarHandler;
use crate::ports::{AvatarStore, UserRepository};
use crate::ApplicationError;
use identity_domain::AvatarRef;
use kernel::Role;

pub struct ReadAvatarService {
    repository: Arc<dyn UserRepository>,
    store: Arc<dyn AvatarStore>,
}

impl ReadAvatarService {
    pub fn new(repository: Arc<dyn UserRepository>, store: Arc<dyn AvatarStore>) -> Self {
        Self { repository, store }
    }
}

#[async_trait::async_trait]
impl ReadAvatarHandler for ReadAvatarService {
    async fn handle(&self, command: ReadAvatar) -> Result<(Vec<u8>, AvatarRef), ApplicationError> {
        if command.reader.user_id() != command.requested && command.reader.role() != Role::Admin {
            return Err(ApplicationError::Forbidden);
        }

        let user = self
            .repository
            .find_by_id(command.requested)
            .await
            .map_err(ApplicationError::from)?
            .ok_or(ApplicationError::UserNotFound)?;
        let avatar = user.avatar().ok_or(ApplicationError::UserNotFound)?;

        let bytes = self
            .store
            .read(command.requested, &avatar)
            .await
            .map_err(|_| ApplicationError::UserNotFound)?;

        Ok((bytes, avatar))
    }
}

#[cfg(test)]
#[path = "read_avatar_service.tests.rs"]
mod tests;
