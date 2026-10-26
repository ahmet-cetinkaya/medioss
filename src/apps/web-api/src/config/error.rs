//! Failures that abort startup before a `Config` exists.
//!
//! Re-exported as `crate::config::ConfigError`.

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error(
        "MEDIOSS_DATA_DIR is required: set it to the writable directory that will hold medioss.db and media/"
    )]
    MissingDataDir,

    #[error("SESSION_SECRET is required and must not be blank")]
    MissingSessionSecret,

    #[error("SESSION_TIMEOUT must be a positive duration in <n>d|h|m|s format, got `{value}`")]
    InvalidSessionTimeout { value: String },

    #[error("MEDIOSS_ORIGIN is required and must not be blank")]
    MissingMediossOrigin,

    #[error("MEDIOSS_ORIGIN must be a single absolute http:// or https:// origin, got `{value}`")]
    InvalidMediossOrigin { value: String },

    #[error(
        "MEDIOSS_DEV_ORIGIN must be blank or a single absolute http:// or https:// origin, got `{value}`"
    )]
    InvalidDevOrigin { value: String },

    #[error("SESSION_COOKIE_SECURE must be exactly `true` or `false`, got `{value}`")]
    InvalidSessionCookieSecure { value: String },

    #[error("PORT must be a number between 0 and 65535, got `{value}`")]
    InvalidPort { value: String },

    #[error("{variable} must be a whole number of bytes between 1 and {maximum}, got `{value}`")]
    InvalidUploadLimit {
        variable: &'static str,
        maximum: u64,
        value: String,
    },

    #[error("MEDIOSS_BIND must be an IP address such as 0.0.0.0 or 127.0.0.1, got `{value}`")]
    InvalidBindHost { value: String },
}
