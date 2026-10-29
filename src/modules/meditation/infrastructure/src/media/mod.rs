//! ffmpeg-backed media adapters: format sniffing, Opus conversion and upload staging.
mod converter;
mod staged_upload;
mod validator;

pub use converter::FfmpegOpusConverter;
pub use staged_upload::{resolve_staged_path, TempFileStaging};
pub use validator::FfmpegAudioValidator;
