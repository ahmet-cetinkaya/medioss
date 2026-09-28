use crate::{
    ports::{Clock, MixRepository, NewMix, SoundRepository},
    ApplicationError,
};
use kernel::UserId;
use soundscape_domain::{valid_name, MixItem};

fn validate(items: &[MixItem]) -> Result<(), ApplicationError> {
    if items.is_empty() || items.len() > soundscape_domain::MAX_MIX_SOUNDS {
        return Err(ApplicationError::InvalidInput("mix items"));
    }
    let mut ids = std::collections::HashSet::new();
    for (position, item) in items.iter().enumerate() {
        if item.sound_id <= 0
            || !ids.insert(item.sound_id)
            || !(0..=100).contains(&item.volume)
            || item.position != position as i64
        {
            return Err(ApplicationError::InvalidInput("mix items"));
        }
    }
    Ok(())
}
async fn ensure_sounds(
    sounds: &dyn SoundRepository,
    items: &[MixItem],
) -> Result<(), ApplicationError> {
    for item in items {
        if sounds
            .find(item.sound_id)
            .await
            .map_err(ApplicationError::Repository)?
            .is_none()
        {
            return Err(ApplicationError::NotFound);
        }
    }
    Ok(())
}
pub async fn create_mix(
    repo: &dyn MixRepository,
    sounds: &dyn SoundRepository,
    clock: &dyn Clock,
    user_id: UserId,
    name: &str,
    items: Vec<MixItem>,
) -> Result<soundscape_domain::Mix, ApplicationError> {
    valid_name(name).map_err(|_| ApplicationError::InvalidInput("mix name"))?;
    validate(&items)?;
    ensure_sounds(sounds, &items).await?;
    repo.create(NewMix {
        user_id,
        name: name.trim().into(),
        items,
        now: clock.now().await,
    })
    .await
    .map_err(ApplicationError::Repository)
}
pub async fn update_mix(
    repo: &dyn MixRepository,
    sounds: &dyn SoundRepository,
    clock: &dyn Clock,
    id: i64,
    user_id: UserId,
    name: &str,
    items: Vec<MixItem>,
) -> Result<soundscape_domain::Mix, ApplicationError> {
    valid_name(name).map_err(|_| ApplicationError::InvalidInput("mix name"))?;
    validate(&items)?;
    ensure_sounds(sounds, &items).await?;
    repo.replace(
        id,
        NewMix {
            user_id,
            name: name.trim().into(),
            items,
            now: clock.now().await,
        },
    )
    .await
    .map_err(ApplicationError::Repository)?
    .ok_or(ApplicationError::NotFound)
}
pub async fn delete_mix(
    repo: &dyn MixRepository,
    id: i64,
    user_id: UserId,
) -> Result<(), ApplicationError> {
    if repo
        .delete_for(id, user_id)
        .await
        .map_err(ApplicationError::Repository)?
    {
        Ok(())
    } else {
        Err(ApplicationError::NotFound)
    }
}
