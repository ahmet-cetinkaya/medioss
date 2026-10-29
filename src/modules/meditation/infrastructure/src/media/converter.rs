//! ffmpeg subprocess conversion. The command line is fixed argv; media is
//! handed to ffmpeg and ffprobe as staged file paths, never through memory.
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use meditation_application::ports::{
    AudioFormat, ConvertError, OpusConverter, StagedMedia, StoredAudio,
};
use tokio::process::Command;

use super::staged_upload::resolve_staged_path;

/// Converts accepted uploads to their stored form: compressed inputs
/// (MP3, Ogg Opus) pass through byte-identical and PCM inputs (WAV,
/// Ogg Vorbis) are compressed losslessly to FLAC.
#[derive(Debug, Clone)]
pub struct FfmpegOpusConverter {
    staging_root: PathBuf,
}

const FFMPEG: &str = "ffmpeg";
const FFPROBE: &str = "ffprobe";
const FLAC_MAGIC: &[u8] = b"fLaC";
const CONVERTED_FILE_PREFIX: &str = "converted-";

impl FfmpegOpusConverter {
    /// `staging_root` is the directory the upload staging writes into, so
    /// converted output lands beside its input.
    pub fn new(staging_root: PathBuf) -> Self {
        Self { staging_root }
    }

    /// Compresses the staged PCM file to FLAC in a new staged file. The
    /// output is deleted again on any failure.
    async fn encode_to_flac(&self, input: &Path) -> Result<StagedMedia, ConvertError> {
        let output = tempfile::Builder::new()
            .prefix(CONVERTED_FILE_PREFIX)
            .suffix(".flac")
            .tempfile_in(&self.staging_root)
            .map_err(|_| ConvertError::Unavailable)?
            .into_temp_path();
        let encoded = Command::new(FFMPEG)
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-y")
            .arg("-i")
            .arg(input)
            .arg("-c:a")
            .arg("flac")
            .arg("-compression_level")
            .arg("8")
            // FLAC stream durations are only written when the muxer sees a
            // seekable output, which a file is and a pipe is not.
            .arg("-f")
            .arg("flac")
            .arg(&output)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            // A missing encoder is an operator problem, not bad media; the
            // API maps it to 503 later.
            .map_err(|_| ConvertError::Unavailable)?;
        if !encoded.status.success() {
            // The full decode gate: a file whose magic passed but whose stream
            // breaks fails here, so nothing is stored (spec FR-004).
            return Err(ConvertError::Decode);
        }
        if !starts_with_flac_magic(&output).await {
            return Err(ConvertError::Decode);
        }
        let size_bytes = tokio::fs::metadata(&output)
            .await
            .map_err(|_| ConvertError::Unavailable)?
            .len();
        let kept = output.keep().map_err(|_| ConvertError::Unavailable)?;
        let address = kept
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or(ConvertError::Unavailable)?;
        Ok(StagedMedia {
            address,
            size_bytes,
        })
    }

    fn path_of(&self, media: &StagedMedia) -> Result<PathBuf, ConvertError> {
        resolve_staged_path(&self.staging_root, &media.address)
            .map_err(|_| ConvertError::Unavailable)
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

/// Probes the staged file for its duration; a probe failure is not fatal to
/// the upload, it just leaves progress unknown (`0`). `container` names the
/// ffprobe demuxer because staged files carry no extension to sniff from.
async fn probe_duration_ms(path: &Path, container: &str) -> i64 {
    let output = Command::new(FFPROBE)
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg("-f")
        .arg(container)
        .arg("-i")
        .arg(path)
        .output()
        .await;
    let output = match output {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            tracing::warn!(status = %output.status, "ffprobe failed; duration stored as 0");
            return 0;
        }
        Err(error) => {
            tracing::warn!(%error, "ffprobe could not run; duration stored as 0");
            return 0;
        }
    };
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .map(|seconds| (seconds * 1000.0).round() as i64)
        .unwrap_or_else(|_| {
            tracing::warn!("ffprobe reported no duration; duration stored as 0");
            0
        })
}

#[async_trait]
impl OpusConverter for FfmpegOpusConverter {
    async fn convert(
        &self,
        media: StagedMedia,
        format: AudioFormat,
    ) -> Result<(StagedMedia, i64, StoredAudio), ConvertError> {
        let input = self.path_of(&media)?;
        let (stored, container, converted) = match format {
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
        let duration_ms = probe_duration_ms(&converted_path, container).await;
        Ok((converted, duration_ms, stored))
    }
}

#[cfg(test)]
#[path = "converter.tests.rs"]
mod tests;
