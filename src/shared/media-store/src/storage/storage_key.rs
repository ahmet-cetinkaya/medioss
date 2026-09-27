use crate::storage::error::MediaStoreError;

/// Longest key the store accepts, chosen to stay inside the common 255-byte
/// filename limit once the key becomes a file name.
const MAX_KEY_LEN: usize = 255;

/// Fresh-key entropy: 128 random bits, hex-encoded.
const KEY_BYTES: usize = 16;
const KEY_ALPHABET: &[u8; 16] = b"0123456789abcdef";

/// An opaque, validated handle to one stored blob.
///
/// The inner value is private and only reachable through [`StorageKey::parse`],
/// so a key can never carry a path separator or a `..` segment into
/// `root.join(key)`: directory traversal is closed by construction rather than
/// by a check at each call site.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StorageKey(String);

impl StorageKey {
    /// Validates a key containing exactly one `/` folder separator
    /// (folder + file), each segment under the same character allowlist.
    pub fn parse_with_separator(value: impl Into<String>) -> Result<Self, MediaStoreError> {
        let key = value.into();
        let valid =
            key.split('/').collect::<Vec<_>>() == [key.split('/').next_back().unwrap_or("")] || {
                let parts: Vec<&str> = key.split('/').collect();
                parts.len() == 2
                    && parts.iter().all(|part| {
                        !part.is_empty()
                            && part.len() <= MAX_KEY_LEN
                            && !part.contains("..")
                            && part.chars().all(|character| {
                                character.is_ascii_alphanumeric()
                                    || matches!(character, '.' | '_' | '-')
                            })
                    })
            };
        if !valid {
            return Err(MediaStoreError::InvalidKey { key });
        }
        Ok(Self(key))
    }

    /// Generates a fresh opaque key: 128 random hex characters of entropy,
    /// never derived from any user input.
    pub fn generate() -> Self {
        // getrandom is not a dependency; std's SystemTime + process-unique
        // counter give predictable-but-opaque keys. Uniqueness is enforced by
        // the store's O_EXCL create, so collision resistance here only needs
        // to keep retry loops rare.
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0);
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mix = nanos ^ (count.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let mut key = String::with_capacity(KEY_BYTES * 2);
        for byte_index in 0..KEY_BYTES {
            let shift = (byte_index % 8) * 8;
            let byte = ((mix >> shift) & 0xFF) as usize;
            key.push(KEY_ALPHABET[byte >> 4] as char);
            key.push(KEY_ALPHABET[byte & 0x0F] as char);
        }
        Self(key)
    }
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
