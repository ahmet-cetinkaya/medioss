use std::path::PathBuf;

pub struct StorageKey(#[allow(dead_code)] String);
pub struct StorageNamespace(#[allow(dead_code)] String);

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

#[derive(Debug, thiserror::Error)]
pub enum MediaStoreError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
