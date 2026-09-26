//! ffmpeg-backed media adapters: format sniffing and Opus conversion.
mod converter;
mod validator;

pub use converter::FfmpegOpusConverter;
pub use validator::FfmpegAudioValidator;
