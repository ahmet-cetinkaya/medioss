#[derive(Debug, thiserror::Error)]
pub enum MediaStoreError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
