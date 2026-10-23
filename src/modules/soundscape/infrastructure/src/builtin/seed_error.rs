#[derive(thiserror::Error, Debug)]
pub enum SeedError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("manifest: {0}")]
    Manifest(String),
    #[error("db: {0}")]
    Db(String),
    #[error("media store: {0}")]
    Media(String),
}
