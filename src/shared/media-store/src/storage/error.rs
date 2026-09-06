#[derive(Debug, thiserror::Error)]
pub enum MediaStoreError {
    #[error("invalid storage key: {key}")]
    InvalidKey { key: String },
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
