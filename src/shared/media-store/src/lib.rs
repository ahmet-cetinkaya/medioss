//! Shared blob storage under $MEDIOSS_DATA_DIR/media.
pub mod storage;

pub use storage::{MediaStore, MediaStoreError, MediaStream, StorageKey};
