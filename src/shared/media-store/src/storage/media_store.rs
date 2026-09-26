use std::path::PathBuf;

use crate::storage::{
    error::MediaStoreError, media_stream::MediaStream, storage_key::StorageKey,
    storage_namespace::StorageNamespace,
};

/// A filesystem blob store: writes, opens, and deletes opaque-keyed files
/// under `root/<namespace>/`. Keys are generated here and validated by
/// [`StorageKey::parse`] on the way back in, so no user-controlled string
/// ever reaches a resolved path.
#[derive(Debug, Clone)]
pub struct MediaStore {
    root: PathBuf,
}

impl MediaStore {
    /// Builds a store rooted at `root` (typically `$MEDIOSS_DATA_DIR/media`).
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn blob_path(&self, ns: StorageNamespace, key: &StorageKey) -> PathBuf {
        self.root.join(ns.as_str()).join(key.as_str())
    }

    /// Persists `bytes` under a freshly generated key and returns it.
    pub async fn put(
        &self,
        ns: StorageNamespace,
        bytes: &[u8],
    ) -> Result<StorageKey, MediaStoreError> {
        let dir = self.root.join(ns.as_str());
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|source| MediaStoreError::Io {
                path: dir.display().to_string(),
                source,
            })?;

        // Loop until the generated key collides with nothing: 128 bits of
        // randomness make a second iteration astronomically unlikely.
        loop {
            let key = StorageKey::generate();
            let final_path = self.blob_path(ns, &key);
            let temp_path = self.blob_path(ns, &key).with_extension("tmp");
            // O_EXCL-style create: never clobber an existing blob.
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
                    // Collision on the temp name: regenerate and retry.
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

    /// Opens a stored blob for reading.
    pub async fn open(
        &self,
        ns: StorageNamespace,
        key: &StorageKey,
    ) -> Result<MediaStream, MediaStoreError> {
        let path = self.blob_path(ns, key);
        let file = tokio::fs::File::open(&path).await.map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                MediaStoreError::NotFound {
                    namespace: ns,
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

    /// Deletes a stored blob; an absent key is reported as [`MediaStoreError::NotFound`].
    pub async fn delete(
        &self,
        ns: StorageNamespace,
        key: &StorageKey,
    ) -> Result<(), MediaStoreError> {
        let path = self.blob_path(ns, key);
        tokio::fs::remove_file(&path).await.map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                MediaStoreError::NotFound {
                    namespace: ns,
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
