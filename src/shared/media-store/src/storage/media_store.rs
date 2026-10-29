use std::path::{Path, PathBuf};
use tokio::io::AsyncReadExt;

use crate::storage::{error::MediaStoreError, media_stream::MediaStream, storage_key::StorageKey};

/// Sanitizes a caller-supplied name segment into a path-safe slug: lowercase
/// ASCII alphanumerics and dashes. The store trusts nobody: whatever the
/// caller passes, only characters from its own allowlist reach the path.
fn slugify(input: &str, max_len: usize) -> String {
    let folded = input
        .chars()
        .flat_map(|character| character.to_lowercase())
        .map(|character| match character {
            'ç' => 'c',
            'ğ' => 'g',
            'ı' => 'i',
            'ö' => 'o',
            'ş' => 's',
            'ü' => 'u',
            '_' => '_',
            other if other.is_ascii_alphanumeric() => other,
            _ => '-',
        })
        .collect::<String>();
    let mut slug = String::new();
    let mut previous_dash = false;
    for character in folded.chars() {
        if character == '-' {
            if !previous_dash && !slug.is_empty() {
                slug.push('-');
                previous_dash = true;
            }
        } else {
            slug.push(character);
            previous_dash = false;
        }
        if slug.chars().count() >= max_len {
            break;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    slug
}

/// Renames `from` over `to`, copying and deleting when the two paths sit on
/// different devices.
async fn move_file(from: &Path, to: &Path) -> std::io::Result<()> {
    match tokio::fs::rename(from, to).await {
        Err(error) if error.kind() == std::io::ErrorKind::CrossesDevices => {
            tokio::fs::copy(from, to).await?;
            tokio::fs::remove_file(from).await
        }
        other => other,
    }
}

/// A filesystem blob store: writes, opens, moves, and deletes files under
/// `root/`. Keys are full relative paths produced here and validated by
/// [`StorageKey::parse`] on the way back in, so no user-controlled string
/// ever reaches a resolved path unfiltered.
#[derive(Debug, Clone)]
pub struct MediaStore {
    root: PathBuf,
}

/// Suffix entropy: 24 bits, hex-encoded (the store retries on collision).
const SUFFIX_LEN: usize = 6;

impl MediaStore {
    /// Builds a store rooted at `root` (typically `$MEDIOSS_DATA_DIR/media`).
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn blob_path(&self, key: &StorageKey) -> PathBuf {
        self.root.join(key.as_str())
    }

    /// Generates a fresh random hex suffix.
    fn random_suffix() -> String {
        // Same construction as StorageKey::generate: time + counter mix.
        // Uniqueness is enforced by O_EXCL; the suffix only keeps retries rare.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0);
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mix = nanos ^ (count.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let tail = mix as u32 as u64 ^ (mix >> 32);
        format!("{tail:0width$x}", width = SUFFIX_LEN)
    }

    /// Persists `bytes` under `folder/{slug}-{suffix}.{ext}` and returns the
    /// full relative key. Both `folder` and `stem` are slugified here — the
    /// path is built entirely from the store's own allowlist.
    pub async fn put(
        &self,
        folder: &str,
        stem: &str,
        ext: &str,
        bytes: &[u8],
    ) -> Result<StorageKey, MediaStoreError> {
        let folder = slugify(folder, 48);
        let stem = slugify(stem, 48);
        let ext: String = ext
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .collect();
        let dir = self.root.join(&folder);
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|source| MediaStoreError::Io {
                path: dir.display().to_string(),
                source,
            })?;

        loop {
            let suffix = Self::random_suffix();
            let key = format!(
                "{folder}/{}-{suffix}.{ext}",
                if stem.is_empty() {
                    "file"
                } else {
                    stem.as_str()
                }
            );
            let key = StorageKey::parse_with_separator(&key)?;
            let final_path = self.blob_path(&key);
            let temp_path = final_path.with_extension("tmp");
            let file = tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp_path)
                .await;
            match file {
                Ok(mut file) => {
                    if let Err(source) = tokio::io::AsyncWriteExt::write_all(&mut file, bytes).await
                    {
                        let _ = tokio::fs::remove_file(&temp_path).await;
                        return Err(MediaStoreError::Io {
                            path: temp_path.display().to_string(),
                            source,
                        });
                    }
                    if let Err(source) = tokio::fs::rename(&temp_path, &final_path).await {
                        let _ = tokio::fs::remove_file(&temp_path).await;
                        return Err(MediaStoreError::Io {
                            path: final_path.display().to_string(),
                            source,
                        });
                    }
                    return Ok(key);
                }
                Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => {
                    let _ = tokio::fs::remove_file(&temp_path).await;
                    continue;
                }
                Err(source) => {
                    return Err(MediaStoreError::Io {
                        path: temp_path.display().to_string(),
                        source,
                    });
                }
            }
        }
    }

    /// Moves the file at `from` into the store under a fresh
    /// `folder/{slug}-{suffix}.{ext}` key, consuming it without reading it
    /// into memory. Uses a rename and falls back to copy-then-delete when
    /// `from` lives on another device. Naming and key validation match
    /// [`MediaStore::put`].
    pub async fn move_in(
        &self,
        from: &Path,
        folder: &str,
        stem: &str,
        ext: &str,
    ) -> Result<StorageKey, MediaStoreError> {
        let (key, target) = self.claim_new_key(folder, stem, ext).await?;
        if let Err(source) = move_file(from, &target).await {
            let _ = tokio::fs::remove_file(&target).await;
            return Err(MediaStoreError::Io {
                path: from.display().to_string(),
                source,
            });
        }
        Ok(key)
    }

    /// Reserves an unused key by creating its empty file, so a concurrent
    /// writer cannot take the same name before the data lands.
    async fn claim_new_key(
        &self,
        folder: &str,
        stem: &str,
        ext: &str,
    ) -> Result<(StorageKey, PathBuf), MediaStoreError> {
        let folder = slugify(folder, 48);
        let stem = slugify(stem, 48);
        let ext: String = ext
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .collect();
        let dir = self.root.join(&folder);
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|source| MediaStoreError::Io {
                path: dir.display().to_string(),
                source,
            })?;
        loop {
            let suffix = Self::random_suffix();
            let name = if stem.is_empty() {
                "file"
            } else {
                stem.as_str()
            };
            let key = StorageKey::parse_with_separator(format!("{folder}/{name}-{suffix}.{ext}"))?;
            let path = self.blob_path(&key);
            match tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .await
            {
                Ok(_) => return Ok((key, path)),
                Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(source) => {
                    return Err(MediaStoreError::Io {
                        path: path.display().to_string(),
                        source,
                    })
                }
            }
        }
    }

    /// Moves a blob from one key to another (same store). Used when a
    /// program folder goes away and its meditations relocate.
    pub async fn rename(
        &self,
        from: &StorageKey,
        to_folder: &str,
        to_stem: &str,
        to_ext: &str,
    ) -> Result<StorageKey, MediaStoreError> {
        let source_path = self.blob_path(from);
        let folder = slugify(to_folder, 48);
        let stem = slugify(to_stem, 48);
        let ext: String = to_ext
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .collect();
        let dir = self.root.join(&folder);
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|source| MediaStoreError::Io {
                path: dir.display().to_string(),
                source,
            })?;
        loop {
            let suffix = Self::random_suffix();
            let key = format!(
                "{folder}/{}-{suffix}.{ext}",
                if stem.is_empty() {
                    "file"
                } else {
                    stem.as_str()
                }
            );
            let key = StorageKey::parse_with_separator(&key)?;
            let target = self.blob_path(&key);
            // `rename` alone would silently overwrite an existing target on
            // Unix (no O_EXCL-equivalent), so first claim the target path
            // with `create_new` — a genuine collision is caught here, before
            // any data moves, instead of destroying whatever already lived
            // at that key.
            match tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target)
                .await
            {
                Ok(_) => {}
                Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(source) => {
                    return Err(MediaStoreError::Io {
                        path: target.display().to_string(),
                        source,
                    });
                }
            }
            if let Err(source) = tokio::fs::rename(&source_path, &target).await {
                let _ = tokio::fs::remove_file(&target).await;
                return Err(MediaStoreError::Io {
                    path: source_path.display().to_string(),
                    source,
                });
            }
            return Ok(key);
        }
    }

    /// Removes a folder and everything under it (a program folder on
    /// program delete). Absent folders are not an error.
    pub async fn delete_folder(&self, folder: &str) -> Result<(), MediaStoreError> {
        let folder = slugify(folder, 48);
        if folder.is_empty() {
            // An empty slug would resolve to the store root itself
            // (`root.join("")` == `root`); refuse rather than wipe every
            // folder under management.
            return Err(MediaStoreError::InvalidKey { key: folder });
        }
        let path = self.root.join(&folder);
        match tokio::fs::remove_dir_all(&path).await {
            Ok(()) => Ok(()),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(MediaStoreError::Io {
                path: path.display().to_string(),
                source,
            }),
        }
    }

    /// Returns a stored blob's length after validating and resolving its opaque key.
    pub async fn size(&self, key: &StorageKey) -> Result<u64, MediaStoreError> {
        let path = self.blob_path(key);
        tokio::fs::metadata(&path)
            .await
            .map(|metadata| metadata.len())
            .map_err(|source| {
                if source.kind() == std::io::ErrorKind::NotFound {
                    MediaStoreError::NotFound {
                        key: key.as_str().to_owned(),
                    }
                } else {
                    MediaStoreError::Io {
                        path: path.display().to_string(),
                        source,
                    }
                }
            })
    }

    /// Opens a stored blob for reading.
    pub async fn open(&self, key: &StorageKey) -> Result<MediaStream, MediaStoreError> {
        let path = self.blob_path(key);
        let file = tokio::fs::File::open(&path).await.map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                MediaStoreError::NotFound {
                    key: key.as_str().to_owned(),
                }
            } else {
                MediaStoreError::Io {
                    path: path.display().to_string(),
                    source,
                }
            }
        })?;
        Ok(MediaStream::new(file))
    }

    /// Opens a byte window from a stored blob without buffering it in memory.
    pub async fn open_range(
        &self,
        key: &StorageKey,
        start: u64,
        length: u64,
    ) -> Result<MediaStream, MediaStoreError> {
        use tokio::io::{AsyncSeekExt, BufReader};

        let path = self.blob_path(key);
        let mut file = tokio::fs::File::open(&path).await.map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                MediaStoreError::NotFound {
                    key: key.as_str().to_owned(),
                }
            } else {
                MediaStoreError::Io {
                    path: path.display().to_string(),
                    source,
                }
            }
        })?;
        file.seek(std::io::SeekFrom::Start(start))
            .await
            .map_err(|source| MediaStoreError::Io {
                path: path.display().to_string(),
                source,
            })?;
        Ok(MediaStream::new(BufReader::new(file).take(length)))
    }

    /// Deletes a stored blob; an absent key is reported as
    /// [`MediaStoreError::NotFound`].
    pub async fn delete(&self, key: &StorageKey) -> Result<(), MediaStoreError> {
        let path = self.blob_path(key);
        tokio::fs::remove_file(&path).await.map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                MediaStoreError::NotFound {
                    key: key.as_str().to_owned(),
                }
            } else {
                MediaStoreError::Io {
                    path: path.display().to_string(),
                    source,
                }
            }
        })
    }
}

#[cfg(test)]
#[path = "media_store.tests.rs"]
mod tests;
