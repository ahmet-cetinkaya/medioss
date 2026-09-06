//! Startup filesystem checks.
//!
//! The data directory holds `medioss.db` and `media/`, so an unwritable or
//! uncreatable path must abort startup instead of failing on the first write.

use std::path::Path;

pub use crate::bootstrap_error::BootstrapError;

const WRITE_CHECK_FILE: &str = ".medioss-write-check";

/// Creates the data directory if missing and proves it is writable.
pub fn ensure_data_dir(dir: &Path) -> Result<(), BootstrapError> {
    std::fs::create_dir_all(dir).map_err(|source| BootstrapError::NotCreatable {
        dir: dir.display().to_string(),
        source,
    })?;

    let probe = dir.join(WRITE_CHECK_FILE);
    std::fs::write(&probe, b"")
        .and_then(|()| std::fs::remove_file(&probe))
        .map_err(|source| BootstrapError::NotWritable {
            dir: dir.display().to_string(),
            source,
        })
}
