use crate::mix::MAX_MIX_SOUNDS;
use crate::name::MAX_NAME_LENGTH;
use crate::sound_icon::MAX_ICON_KEY_LENGTH;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("name must be non-empty and at most {MAX_NAME_LENGTH} characters")]
    InvalidName,
    #[error("sound size and duration must be positive")]
    InvalidMedia,
    #[error("sound storage key must be non-empty")]
    InvalidStorageKey,
    #[error("sound content type is not a supported audio type")]
    InvalidContentType,
    #[error("icon key must be at most {MAX_ICON_KEY_LENGTH} characters of a-z, 0-9 or '-'")]
    InvalidIconKey,
    #[error("mix may contain at most {MAX_MIX_SOUNDS} distinct sounds")]
    InvalidMixItems,
    #[error("mix item volume must be between 0 and 100")]
    InvalidVolume,
    #[error("mix positions must be contiguous from zero")]
    InvalidPosition,
}
