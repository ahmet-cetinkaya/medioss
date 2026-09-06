//! Failures raised while proving the data directory is usable.
//!
//! Re-exported as `crate::bootstrap::BootstrapError`.

#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    #[error("MEDIOSS_DATA_DIR `{dir}` could not be created")]
    NotCreatable {
        dir: String,
        #[source]
        source: std::io::Error,
    },

    #[error("MEDIOSS_DATA_DIR `{dir}` is not writable")]
    NotWritable {
        dir: String,
        #[source]
        source: std::io::Error,
    },
}
