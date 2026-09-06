use crate::storage::error::MediaStoreError;

/// Longest key the store accepts, chosen to stay inside the common 255-byte
/// filename limit once the key becomes a file name.
const MAX_KEY_LEN: usize = 255;

/// An opaque, validated handle to one stored blob.
///
/// The inner value is private and only reachable through [`StorageKey::parse`],
/// so a key can never carry a path separator or a `..` segment into
/// `root.join(key)`: directory traversal is closed by construction rather than
/// by a check at each call site.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StorageKey(String);

impl StorageKey {
    /// Validates an untrusted key, whether freshly generated or read back from
    /// a database row.
    pub fn parse(value: impl Into<String>) -> Result<Self, MediaStoreError> {
        let key = value.into();

        let is_valid = !key.is_empty()
            && key.len() <= MAX_KEY_LEN
            && !key.contains("..")
            && key.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
            });

        if !is_valid {
            return Err(MediaStoreError::InvalidKey { key });
        }

        Ok(Self(key))
    }

    /// The validated key, for persistence and for building the on-disk path.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
#[path = "storage_key.tests.rs"]
mod tests;
