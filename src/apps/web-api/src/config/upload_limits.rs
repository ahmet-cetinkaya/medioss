//! Per-kind upload size caps, read once at startup and injected into the modules.
//!
//! Re-exported as `crate::config::UploadLimits`.

pub(super) const DEFAULT_MAX_UPLOAD_BYTES: usize = 100 * 1024 * 1024;
pub(super) const DEFAULT_MAX_SOUND_UPLOAD_BYTES: usize = 100 * 1024 * 1024;
pub(super) const DEFAULT_MAX_COVER_BYTES: usize = 5 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UploadLimits {
    pub meditation_audio_bytes: usize,
    pub sound_audio_bytes: usize,
    pub program_cover_bytes: usize,
}

impl Default for UploadLimits {
    fn default() -> Self {
        Self {
            meditation_audio_bytes: DEFAULT_MAX_UPLOAD_BYTES,
            sound_audio_bytes: DEFAULT_MAX_SOUND_UPLOAD_BYTES,
            program_cover_bytes: DEFAULT_MAX_COVER_BYTES,
        }
    }
}

#[cfg(test)]
#[path = "upload_limits.tests.rs"]
mod tests;
