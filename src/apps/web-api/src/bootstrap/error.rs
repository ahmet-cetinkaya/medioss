//! Failures raised while preparing persistent application data.
//!
//! Re-exported as `crate::bootstrap::BootstrapError`.

use std::path::PathBuf;

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

    #[error("database `{}` could not be opened", path.display())]
    DatabaseOpen {
        path: PathBuf,
        #[source]
        source: sqlx::Error,
    },

    #[error("database migration failed for `{}`", path.display())]
    Migration {
        path: PathBuf,
        #[source]
        source: sea_orm::DbErr,
    },
}
