//! Soundscape domain entities and invariants.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const STORED_CONTENT_TYPE: &str = "audio/ogg";

/// Content types a persisted sound may carry. Compressed uploads keep their
/// original format; PCM-derived ones are stored losslessly as FLAC.
pub fn valid_content_type(content_type: &str) -> bool {
    matches!(
        content_type,
        "audio/ogg" | "audio/mpeg" | "audio/flac" | "audio/wav"
    )
}
pub const MAX_NAME_LENGTH: usize = 80;
/// Maximum number of distinct sounds one mix may contain. Deliberately
/// generous: layering many ambient sounds is a supported use, and the only
/// ceiling is a sanity bound against runaway payloads, not a product limit.
pub const MAX_MIX_SOUNDS: usize = 64;

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
    #[error("mix may contain at most {MAX_MIX_SOUNDS} distinct sounds")]
    InvalidMixItems,
    #[error("mix item volume must be between 0 and 100")]
    InvalidVolume,
    #[error("mix positions must be contiguous from zero")]
    InvalidPosition,
}

pub fn valid_name(name: &str) -> Result<String, DomainError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME_LENGTH {
        return Err(DomainError::InvalidName);
    }
    Ok(name.to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundCategory {
    pub id: i64,
    pub name: String,
    pub slug: String,
}
impl SoundCategory {
    pub fn new(id: i64, name: &str, slug: &str) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            name: valid_name(name)?,
            slug: valid_name(slug)?,
        })
    }
}

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
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MixItem {
    pub sound_id: i64,
    pub volume: i64,
    pub position: i64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mix {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub items: Vec<MixItem>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl Mix {
    pub fn new(
        id: i64,
        user_id: i64,
        name: &str,
        items: Vec<MixItem>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if user_id <= 0 {
            return Err(DomainError::InvalidMixItems);
        }
        if items.len() > MAX_MIX_SOUNDS {
            return Err(DomainError::InvalidMixItems);
        }
        let mut ids = std::collections::HashSet::new();
        for (index, item) in items.iter().enumerate() {
            if item.sound_id <= 0 || !ids.insert(item.sound_id) {
                return Err(DomainError::InvalidMixItems);
            }
            if !(0..=100).contains(&item.volume) {
                return Err(DomainError::InvalidVolume);
            }
            if item.position != index as i64 {
                return Err(DomainError::InvalidPosition);
            }
        }
        Ok(Self {
            id,
            user_id,
            name: valid_name(name)?,
            items,
            created_at,
            updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mix_invariants_reject_duplicates_and_bad_volume() {
        let now = Utc::now();
        assert!(Mix::new(
            1,
            1,
            "Mix",
            vec![
                MixItem {
                    sound_id: 1,
                    volume: 100,
                    position: 0
                },
                MixItem {
                    sound_id: 1,
                    volume: 50,
                    position: 1
                }
            ],
            now,
            now
        )
        .is_err());
        assert!(Mix::new(
            1,
            1,
            "Mix",
            vec![MixItem {
                sound_id: 1,
                volume: 101,
                position: 0
            }],
            now,
            now
        )
        .is_err());
    }

    #[test]
    fn mix_hydration_tolerates_empty_items_after_sound_deletion() {
        let now = Utc::now();
        let mix = Mix::new(1, 1, "Mix", Vec::new(), now, now).expect("empty mix hydrates");
        assert!(mix.items.is_empty());
    }
}
