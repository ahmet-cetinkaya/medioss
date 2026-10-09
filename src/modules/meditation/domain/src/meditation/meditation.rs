use chrono::{DateTime, Utc};
use thiserror::Error;

/// The stored audio content type for legacy Opus-normalized rows. New rows
/// keep their upload's content type (MP3, Opus) or store FLAC when the
/// source was uncompressed PCM; see [`valid_content_type`].
pub const STORED_CONTENT_TYPE: &str = "audio/ogg";

/// Content types a persisted meditation may carry. Compressed uploads keep
/// their original format; PCM-derived ones are stored losslessly as FLAC.
pub fn valid_content_type(content_type: &str) -> bool {
    matches!(
        content_type,
        "audio/ogg" | "audio/mpeg" | "audio/flac" | "audio/wav"
    )
}

/// Maximum accepted name length, in characters.
const MAX_NAME_LENGTH: usize = 200;

/// Maximum accepted description length, in characters.
pub const MAX_DESCRIPTION_LENGTH: usize = 2000;

/// Why a [`Meditation`] could not be constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MeditationError {
    #[error("meditation name must not be empty")]
    InvalidName,
    #[error("meditation size must be positive")]
    InvalidSize,
    #[error("meditation content type is not a supported audio type")]
    InvalidContentType,
    #[error("meditation storage key must not be empty")]
    InvalidStorageKey,
    #[error("description must be at most 2000 characters")]
    InvalidDescription,
}

/// A curated meditation audio entry in the shared library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meditation {
    id: i64,
    name: String,
    content_type: String,
    size_bytes: i64,
    storage_key: String,
    duration_ms: i64,
    created_at: DateTime<Utc>,
    description: Option<String>,
}

impl Meditation {
    /// Constructs a meditation, enforcing the row invariants.
    pub fn new(
        id: i64,
        name: &str,
        content_type: &str,
        size_bytes: i64,
        storage_key: &str,
        duration_ms: i64,
        created_at: DateTime<Utc>,
    ) -> Result<Self, MeditationError> {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > MAX_NAME_LENGTH {
            return Err(MeditationError::InvalidName);
        }
        if !valid_content_type(content_type) {
            return Err(MeditationError::InvalidContentType);
        }
        if size_bytes <= 0 {
            return Err(MeditationError::InvalidSize);
        }
        if storage_key.is_empty() {
            return Err(MeditationError::InvalidStorageKey);
        }

        Ok(Self {
            id,
            name: name.to_owned(),
            content_type: content_type.to_owned(),
            size_bytes,
            storage_key: storage_key.to_owned(),
            duration_ms: duration_ms.max(0),
            created_at,
            description: None,
        })
    }

    /// Returns the meditation with its description replaced. Blank text
    /// clears it (`None`); text over the limit is rejected.
    pub fn with_description(mut self, description: Option<&str>) -> Result<Self, MeditationError> {
        let description = description.map(str::trim).filter(|text| !text.is_empty());
        if description.is_some_and(|text| text.chars().count() > MAX_DESCRIPTION_LENGTH) {
            return Err(MeditationError::InvalidDescription);
        }
        self.description = description.map(str::to_owned);
        Ok(self)
    }

    /// Optional free-text description; never blank.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Row identifier.
    pub fn id(&self) -> i64 {
        self.id
    }

    /// Display name, already trimmed.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Stored audio content type (always `audio/ogg`).
    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    /// Stored byte count of the converted audio.
    pub fn size_bytes(&self) -> i64 {
        self.size_bytes
    }

    /// Media-store key of the stored audio bytes.
    pub fn storage_key(&self) -> &str {
        &self.storage_key
    }

    /// Decoded audio duration in milliseconds; `0` when unknown (probe
    /// failed or the row predates duration capture).
    pub fn duration_ms(&self) -> i64 {
        self.duration_ms
    }

    /// Row creation timestamp.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[cfg(test)]
#[path = "meditation.tests.rs"]
mod tests;
