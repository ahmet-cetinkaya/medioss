use async_trait::async_trait;
use soundscape_application::ports::media::{AudioFormat, OpusConverter, StoredAudio};
use tokio::{io::AsyncWriteExt, process::Command};

pub struct FfmpegOpusConverter;
#[async_trait]
impl OpusConverter for FfmpegOpusConverter {
    async fn convert(
        &self,
        bytes: Vec<u8>,
        format: AudioFormat,
    ) -> Result<(Vec<u8>, i64, StoredAudio), String> {
        let stored = match format {
            // Already compressed uploads are stored byte-identical: any
            // re-encode could only lose quality.
            AudioFormat::OggOpus => StoredAudio {
                extension: "opus",
                content_type: "audio/ogg",
            },
            AudioFormat::Mp3 => StoredAudio {
                extension: "mp3",
                content_type: "audio/mpeg",
            },
            // Uncompressed PCM is compressed losslessly to FLAC so the
            // stored bytes decode back to the exact uploaded samples. The
            // encode targets a temp file (not a pipe) because the FLAC muxer
            // only writes stream durations it can size up front.
            _ => StoredAudio {
                extension: "flac",
                content_type: "audio/flac",
            },
        };
        let output = match format {
            AudioFormat::OggOpus | AudioFormat::Mp3 => bytes,
            _ => {
                let out = tempfile::Builder::new()
                    .suffix(".flac")
                    .tempfile()
                    .map_err(|_| "duration staging unavailable".to_owned())?;
                let out_path = out.path().to_owned();
                let mut command = Command::new("ffmpeg");
                command
                    .args([
                        "-hide_banner",
                        "-loglevel",
                        "error",
                        "-y",
                        "-i",
                        "pipe:0",
                        "-c:a",
                        "flac",
                        "-compression_level",
                        "8",
                    ])
                    .arg(&out_path)
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null());
                let mut child = command
                    .spawn()
                    .map_err(|_| "ffmpeg is unavailable".to_owned())?;
                let mut stdin = child
                    .stdin
                    .take()
                    .ok_or_else(|| "ffmpeg stdin unavailable".to_owned())?;
                let write = tokio::spawn(async move {
                    stdin.write_all(&bytes).await?;
                    stdin.shutdown().await
                });
                let result = child
                    .wait_with_output()
                    .await
                    .map_err(|_| "ffmpeg failed".to_owned())?;
                write
                    .await
                    .map_err(|_| "ffmpeg input failed".to_owned())?
                    .map_err(|_| "ffmpeg input failed".to_owned())?;
                if !result.status.success() {
                    return Err("audio decode failed".to_owned());
                }
                let output = tokio::fs::read(&out_path)
                    .await
                    .map_err(|_| "flac output unreadable".to_owned())?;
                if !output.starts_with(b"fLaC") {
                    return Err("audio decode failed".to_owned());
                }
                output
            }
        };
        let duration = probe_duration(&output, stored.extension).await?;
        Ok((output, duration, stored))
    }
}

/// Stages `bytes` at `path` without duplicating them: `tokio::fs::write` copies
/// the whole slice into an owned buffer first, doubling the memory of a large
/// upload, while `write_all` on a file copies through a small bounded buffer.
async fn stage_bytes(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = tokio::fs::File::create(path).await?;
    file.write_all(bytes).await?;
    file.flush().await
}

async fn probe_duration(bytes: &[u8], extension: &str) -> Result<i64, String> {
    let file = tempfile::Builder::new()
        .suffix(&format!(".{extension}"))
        .tempfile()
        .map_err(|_| "duration staging unavailable".to_owned())?;
    stage_bytes(file.path(), bytes)
        .await
        .map_err(|_| "duration staging failed".to_owned())?;
    let result = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            "-i",
        ])
        .arg(file.path())
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

#[cfg(test)]
#[path = "ffmpeg_opus_converter.tests.rs"]
mod tests;
