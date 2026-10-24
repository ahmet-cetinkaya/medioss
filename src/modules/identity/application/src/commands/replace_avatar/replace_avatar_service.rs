use std::sync::Arc;

use super::replace_avatar::ReplaceAvatar;
use super::replace_avatar_handler::ReplaceAvatarHandler;
use crate::ports::{AvatarStore, UserRepository};
use crate::ApplicationError;
use identity_domain::User;

pub struct ReplaceAvatarService {
    repository: Arc<dyn UserRepository>,
    store: Arc<dyn AvatarStore>,
}

impl ReplaceAvatarService {
    pub fn new(repository: Arc<dyn UserRepository>, store: Arc<dyn AvatarStore>) -> Self {
        Self { repository, store }
    }

    async fn replace_avatar(
        &self,
        actor: &kernel::Actor,
        bytes: Vec<u8>,
    ) -> Result<User, ApplicationError> {
        let (encoded, ext) = self
            .store
            .normalize(bytes)
            .await
            .map_err(ApplicationError::from)?;
        let user = self
            .repository
            .find_by_id(actor.user_id())
            .await
            .map_err(ApplicationError::from)?
            .ok_or(ApplicationError::UserNotFound)?;
        let previous = user.avatar();

        let avatar = self
            .store
            .write(actor.user_id(), ext, &encoded)
            .await
            .map_err(ApplicationError::from)?;
        // `set_profile` can fail after committing its update (while re-reading
        // the user). Keep the new blob so the database can never point at a
        // deleted key; an orphan is purged on user deletion.
        let updated = self
            .repository
            .set_profile(actor.user_id(), user.display_name().cloned(), Some(avatar))
            .await
            .map_err(ApplicationError::from)?;

        if let Some(previous) = previous {
            let _ = self.store.remove(actor.user_id(), &previous).await;
        }

        Ok(updated)
    }
}

#[async_trait::async_trait]
impl ReplaceAvatarHandler for ReplaceAvatarService {
    async fn handle(&self, command: ReplaceAvatar) -> Result<User, ApplicationError> {
        self.replace_avatar(&command.actor, command.bytes).await
    }
}

#[cfg(test)]
#[path = "replace_avatar_service.tests.rs"]
mod tests;
