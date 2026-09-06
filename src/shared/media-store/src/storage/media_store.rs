use std::path::PathBuf;

use crate::storage::{
    error::MediaStoreError, storage_key::StorageKey, storage_namespace::StorageNamespace,
};

pub struct MediaStore {
    #[allow(dead_code)]
    root: PathBuf,
}

impl MediaStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub async fn put(
        &self,
        _ns: &StorageNamespace,
        _bytes: &[u8],
    ) -> Result<StorageKey, MediaStoreError> {
        todo!()
    }

    pub async fn open(
        &self,
        _ns: &StorageNamespace,
        _key: &StorageKey,
    ) -> Result<tokio::fs::File, MediaStoreError> {
        todo!()
    }

    pub async fn delete(
        &self,
        _ns: &StorageNamespace,
        _key: &StorageKey,
    ) -> Result<(), MediaStoreError> {
        todo!()
    }
}
