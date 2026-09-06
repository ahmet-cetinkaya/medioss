//! Failures that abort startup before a `Config` exists.
//!
//! Re-exported as `crate::config::ConfigError`.

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error(
        "MEDIOSS_DATA_DIR is required: set it to the writable directory that will hold medioss.db and media/"
    )]
    MissingDataDir,

    #[error("PORT must be a number between 0 and 65535, got `{value}`")]
    InvalidPort { value: String },
}
