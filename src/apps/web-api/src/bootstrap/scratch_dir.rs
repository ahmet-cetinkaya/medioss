//! Scratch space for media conversion.
//!
//! The container runs with a read-only root filesystem, so the system temp
//! directory is not writable there. ffmpeg staging files and duration probes
//! need one, so scratch files live in a `tmp` folder on the data volume.

use std::path::{Path, PathBuf};

use super::BootstrapError;

/// Folder under the data dir that holds scratch and staged upload files.
pub const SCRATCH_DIR_NAME: &str = "tmp";
const TEMP_DIR_VAR: &str = "TMPDIR";
const WRITE_CHECK_FILE: &str = ".scratch-write-check";

/// Creates the scratch folder, empties it (files left by a crashed run are
/// never reused) and proves it is writable. Returns its path.
pub fn prepare_scratch_dir(data_dir: &Path) -> Result<PathBuf, BootstrapError> {
    let scratch = data_dir.join(SCRATCH_DIR_NAME);
    let failed = |source| BootstrapError::ScratchDirUnusable {
        dir: scratch.display().to_string(),
        source,
    };

    if scratch.exists() {
        std::fs::remove_dir_all(&scratch).map_err(failed)?;
    }
    std::fs::create_dir_all(&scratch).map_err(failed)?;
    let probe = scratch.join(WRITE_CHECK_FILE);
    std::fs::write(&probe, b"").map_err(failed)?;
    std::fs::remove_file(&probe).map_err(failed)?;
    Ok(scratch)
}

/// Prepares the scratch folder and points `TMPDIR` at it, so every temp file,
/// and every ffmpeg or ffprobe child, uses it. Mutates the process
/// environment, so call it before the async runtime starts any thread.
pub fn use_scratch_dir_as_temp_dir(data_dir: &Path) -> Result<(), BootstrapError> {
    let scratch = prepare_scratch_dir(data_dir)?;
    std::env::set_var(TEMP_DIR_VAR, scratch);
    Ok(())
}

#[cfg(test)]
#[path = "scratch_dir.tests.rs"]
mod tests;
