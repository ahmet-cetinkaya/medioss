//! Startup filesystem checks.
//!
//! The data directory holds `medioss.db` and `media/`, so an unwritable or
//! uncreatable path must abort startup instead of failing on the first write.

use std::path::Path;

pub mod database;
pub mod error;

pub use error::BootstrapError;

const WRITE_CHECK_FILE: &str = ".medioss-write-check";

/// Creates the data directory if missing and proves it is writable.
pub fn ensure_data_dir(dir: &Path) -> Result<(), BootstrapError> {
    std::fs::create_dir_all(dir).map_err(|source| BootstrapError::NotCreatable {
        dir: dir.display().to_string(),
        source,
    })?;

    let probe = dir.join(WRITE_CHECK_FILE);
    std::fs::write(&probe, b"").map_err(|source| BootstrapError::NotWritable {
        dir: dir.display().to_string(),
        source,
    })?;

    // The write proved the directory usable, so a failed cleanup is not fatal --
    // reporting it as "not writable" would be a false diagnosis.
    if let Err(err) = std::fs::remove_file(&probe) {
        tracing::warn!("could not remove write probe {}: {err}", probe.display());
    }

    Ok(())
}
