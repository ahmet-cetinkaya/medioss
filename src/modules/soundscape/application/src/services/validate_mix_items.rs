use crate::ApplicationError;
use soundscape_domain::{MixItem, MAX_MIX_SOUNDS};

const MAX_VOLUME: i64 = 100;

pub(crate) fn validate_mix_items(items: &[MixItem]) -> Result<(), ApplicationError> {
    if items.is_empty() || items.len() > MAX_MIX_SOUNDS {
        return Err(ApplicationError::InvalidInput("mix items"));
    }
    let mut ids = std::collections::HashSet::new();
    for (position, item) in items.iter().enumerate() {
        if item.sound_id <= 0
            || !ids.insert(item.sound_id)
            || !(0..=MAX_VOLUME).contains(&item.volume)
            || item.position != position as i64
        {
            return Err(ApplicationError::InvalidInput("mix items"));
        }
    }
    Ok(())
}
