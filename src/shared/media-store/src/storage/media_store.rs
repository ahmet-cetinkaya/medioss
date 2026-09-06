use std::path::PathBuf;

use crate::storage::{
    error::MediaStoreError, media_stream::MediaStream, storage_key::StorageKey,
    storage_namespace::StorageNamespace,
};

/// `Clone` keeps the store usable in `Clone`-required Axum state without an `Arc`.
#[derive(Debug, Clone)]
pub struct MediaStore {
    // M2 reads this when the storage methods stop being `todo!()`.
    #[allow(dead_code)]
    root: PathBuf,
}

impl MediaStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub async fn put(
        &self,
        _ns: StorageNamespace,
        _bytes: &[u8],
    ) -> Result<StorageKey, MediaStoreError> {
        todo!()
    }

    pub async fn open(
        &self,
        _ns: StorageNamespace,
        _key: &StorageKey,
    ) -> Result<MediaStream, MediaStoreError> {
        todo!()
    }

    pub async fn delete(
        &self,
        _ns: StorageNamespace,
        _key: &StorageKey,
    ) -> Result<(), MediaStoreError> {
        todo!()
    }
}
