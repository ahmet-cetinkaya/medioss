//! Filesystem-backed media storage: the store, its addressing types, and its errors.

pub mod error;
pub mod media_store;
pub mod storage_key;
pub mod storage_namespace;

pub use error::MediaStoreError;
pub use media_store::MediaStore;
pub use storage_key::StorageKey;
pub use storage_namespace::StorageNamespace;
