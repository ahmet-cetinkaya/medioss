use crate::error::DomainError;
use crate::name::valid_name;
use crate::sound::valid_content_type;
use crate::sound_icon::SoundIcon;
use chrono::{DateTime, Utc};

pub const STORED_CONTENT_TYPE: &str = "audio/ogg";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sound {
    pub id: i64,
    pub name: String,
    pub category_id: i64,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_key: String,
    pub duration_ms: i64,
    pub created_at: DateTime<Utc>,
    /// True when the row came from the shipped built-in sound set.
    pub builtin: bool,
    /// Key of a bundled UI glyph, if one was chosen.
    pub icon_key: Option<String>,
    /// Admin-uploaded icon image, if any (wins over `icon_key` in the UI).
    pub icon: Option<SoundIcon>,
}
impl Sound {
    #[allow(clippy::too_many_arguments)] // media metadata fields are naturally flat
    pub fn new(
        id: i64,
        name: &str,
        category_id: i64,
        content_type: &str,
        size_bytes: i64,
        storage_key: &str,
        duration_ms: i64,
        created_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if category_id <= 0 || size_bytes <= 0 || duration_ms <= 0 {
            return Err(DomainError::InvalidMedia);
        }
        if !valid_content_type(content_type) {
            return Err(DomainError::InvalidContentType);
        }
        if storage_key.is_empty() {
            return Err(DomainError::InvalidStorageKey);
        }
        Ok(Self {
            id,
            name: valid_name(name)?,
            category_id,
            content_type: content_type.to_owned(),
            size_bytes,
            storage_key: storage_key.to_owned(),
            duration_ms,
            created_at,
            builtin: false,
            icon_key: None,
            icon: None,
        })
    }
}

#[cfg(test)]
#[path = "sound.tests.rs"]
mod tests;
