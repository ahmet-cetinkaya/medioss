//! ffmpeg subprocess conversion. Media is handed to ffmpeg and ffprobe as
//! staged file paths, never through memory. Mirrors meditation-infrastructure's
//! converter; the boundary rule forbids sharing it across modules.
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use soundscape_application::ports::media::{AudioFormat, OpusConverter, StoredAudio};
use soundscape_application::ports::staged_upload::StagedMedia;
use tokio::process::Command;

use super::staged_upload::resolve_staged_path;

const FFMPEG: &str = "ffmpeg";
const FFPROBE: &str = "ffprobe";
const FLAC_MAGIC: &[u8] = b"fLaC";
const CONVERTED_FILE_PREFIX: &str = "converted-";

/// Converts accepted uploads to their stored form: compressed inputs
/// (MP3, Ogg Opus) pass through byte-identical and PCM inputs (WAV,
/// Ogg Vorbis) are compressed losslessly to FLAC.
#[derive(Debug, Clone)]
pub struct FfmpegOpusConverter {
    staging_root: PathBuf,
}

impl FfmpegOpusConverter {
    /// `staging_root` is the directory the upload staging writes into, so
    /// converted output lands beside its input.
    pub fn new(staging_root: PathBuf) -> Self {
        Self { staging_root }
    }

    /// Compresses the staged PCM file to FLAC in a new staged file. The
    /// output is deleted again on any failure.
    async fn encode_to_flac(&self, input: &Path) -> Result<StagedMedia, String> {
        let output = tempfile::Builder::new()
            .prefix(CONVERTED_FILE_PREFIX)
            .suffix(".flac")
            .tempfile_in(&self.staging_root)
            .map_err(|_| "duration staging unavailable".to_owned())?
            .into_temp_path();
        // FLAC stream durations are only written when the muxer sees a
        // seekable output, which a file is and a pipe is not.
        let encoded = Command::new(FFMPEG)
            .args(["-hide_banner", "-loglevel", "error", "-y", "-i"])
            .arg(input)
            .args(["-c:a", "flac", "-compression_level", "8", "-f", "flac"])
            .arg(&output)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await
            .map_err(|_| "ffmpeg is unavailable".to_owned())?;
        if !encoded.success() || !starts_with_flac_magic(&output).await {
            return Err("audio decode failed".to_owned());
        }
        let size_bytes = tokio::fs::metadata(&output)
            .await
            .map_err(|_| "flac output unreadable".to_owned())?
            .len();
        let kept = output
            .keep()
            .map_err(|_| "flac output unreadable".to_owned())?;
        let address = kept
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or_else(|| "flac output unreadable".to_owned())?;
        Ok(StagedMedia {
            address,
            size_bytes,
        })
    }

    fn path_of(&self, media: &StagedMedia) -> Result<PathBuf, String> {
        resolve_staged_path(&self.staging_root, &media.address).map_err(|error| error.to_string())
    }
}

async fn starts_with_flac_magic(path: &Path) -> bool {
    use tokio::io::AsyncReadExt;
    let Ok(mut file) = tokio::fs::File::open(path).await else {
        return false;
    };
    let mut magic = [0u8; FLAC_MAGIC.len()];
    file.read_exact(&mut magic).await.is_ok() && magic == FLAC_MAGIC
}

/// Probes the staged file for its duration in milliseconds. `container` names
/// the ffprobe demuxer because staged files carry no extension to sniff from.
async fn probe_duration_ms(path: &Path, container: &str) -> Result<i64, String> {
    let result = Command::new(FFPROBE)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            "-f",
            container,
            "-i",
        ])
        .arg(path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .await
        .map_err(|_| "ffprobe failed".to_owned())?;
    if !result.status.success() {
        return Err("audio duration unavailable".to_owned());
    }
    let seconds = std::str::from_utf8(&result.stdout)
        .ok()
        .and_then(|value| value.trim().parse::<f64>().ok())
        .filter(|value| *value > 0.0)
        .ok_or_else(|| "audio duration unavailable".to_owned())?;
    Ok((seconds * 1000.0).round() as i64)
}

#[async_trait]
impl OpusConverter for FfmpegOpusConverter {
    async fn convert(
        &self,
        media: StagedMedia,
        format: AudioFormat,
    ) -> Result<(StagedMedia, i64, StoredAudio), String> {
        let input = self.path_of(&media)?;
        let input_address = media.address.clone();
        let (stored, container, converted) = match format {
            // Already compressed uploads are stored byte-identical: any
            // re-encode could only lose quality.
            AudioFormat::OggOpus => (
                StoredAudio {
                    extension: "opus",
                    content_type: "audio/ogg",
                },
                "ogg",
                media,
            ),
            AudioFormat::Mp3 => (
                StoredAudio {
                    extension: "mp3",
                    content_type: "audio/mpeg",
                },
                "mp3",
                media,
            ),
            // Uncompressed PCM is compressed losslessly to FLAC so the
            // stored bytes decode back to the exact uploaded samples.
            _ => (
                StoredAudio {
                    extension: "flac",
                    content_type: "audio/flac",
                },
                "flac",
                self.encode_to_flac(&input).await?,
            ),
        };
        let converted_path = self.path_of(&converted)?;
        match probe_duration_ms(&converted_path, container).await {
            Ok(duration_ms) => Ok((converted, duration_ms, stored)),
            Err(error) => {
                if converted.address != input_address {
                    let _ = tokio::fs::remove_file(&converted_path).await;
                }
                Err(error)
            }
        }
    }
}

#[cfg(test)]
#[path = "ffmpeg_opus_converter.tests.rs"]
mod tests;
