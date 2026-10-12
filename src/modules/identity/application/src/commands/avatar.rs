use identity_domain::AvatarRef;
use kernel::{Actor, UserId};

use crate::{
    ports::{AvatarStore, UserRepository},
    ApplicationError,
};

pub async fn replace_avatar(
    repository: &impl UserRepository,
    store: &dyn AvatarStore,
    actor: &Actor,
    bytes: Vec<u8>,
) -> Result<identity_domain::User, ApplicationError> {
    let (encoded, ext) = store
        .normalize(bytes)
        .await
        .map_err(ApplicationError::from)?;
    let user = repository
        .find_by_id(actor.user_id())
        .await
        .map_err(ApplicationError::from)?
        .ok_or(ApplicationError::UserNotFound)?;
    let previous = user.avatar();

    let avatar = store
        .write(actor.user_id(), ext, &encoded)
        .await
        .map_err(ApplicationError::from)?;
    // `set_profile` can fail after committing its update (while re-reading the
    // user). Keep the new blob so the database can never point at a deleted
    // key; an orphan is purged on user deletion.
    let updated = repository
        .set_profile(actor.user_id(), user.display_name().cloned(), Some(avatar))
        .await
        .map_err(ApplicationError::from)?;

    if let Some(previous) = previous {
        let _ = store.remove(actor.user_id(), &previous).await;
    }

    Ok(updated)
}

pub async fn clear_avatar(
    repository: &impl UserRepository,
    store: &dyn AvatarStore,
    actor: &Actor,
) -> Result<(), ApplicationError> {
    let user = repository
        .find_by_id(actor.user_id())
        .await
        .map_err(ApplicationError::from)?
        .ok_or(ApplicationError::UserNotFound)?;

    if let Some(avatar) = user.avatar() {
        repository
            .set_profile(actor.user_id(), user.display_name().cloned(), None)
            .await
            .map_err(ApplicationError::from)?;
        let _ = store.remove(actor.user_id(), &avatar).await;
    }

    Ok(())
}

pub async fn read_avatar(
    repository: &impl UserRepository,
    store: &dyn AvatarStore,
    requested: UserId,
    reader: &Actor,
) -> Result<(Vec<u8>, AvatarRef), ApplicationError> {
    if reader.user_id() != requested && reader.role() != kernel::Role::Admin {
        return Err(ApplicationError::Forbidden);
    }

    let user = repository
        .find_by_id(requested)
        .await
        .map_err(ApplicationError::from)?
        .ok_or(ApplicationError::UserNotFound)?;
    let avatar = user.avatar().ok_or(ApplicationError::UserNotFound)?;

    let bytes = store
        .read(requested, &avatar)
        .await
        .map_err(|_| ApplicationError::UserNotFound)?;

    Ok((bytes, avatar))
}

#[cfg(test)]
#[path = "avatar.tests.rs"]
mod tests;
