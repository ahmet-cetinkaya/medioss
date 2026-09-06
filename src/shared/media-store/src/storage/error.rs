use crate::storage::storage_namespace::StorageNamespace;

/// Why a store operation failed.
///
/// Deliberately no `#[from] std::io::Error`: an automatic conversion would let
/// `?` discard which path failed. Callers must write
/// `.map_err(|source| MediaStoreError::Io { path, source })`, so every io
/// failure names its file.
#[derive(Debug, thiserror::Error)]
pub enum MediaStoreError {
    #[error("no blob at {}/{}", namespace.as_str(), key)]
    NotFound {
        namespace: StorageNamespace,
        key: String,
    },
    #[error("invalid storage key: {key}")]
    InvalidKey { key: String },
    #[error("io on {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
}
