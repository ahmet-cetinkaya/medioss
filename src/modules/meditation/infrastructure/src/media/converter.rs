//! ffmpeg subprocess Opus conversion. The command line is fixed argv —
//! media bytes flow through pipes, never through arguments.
use async_trait::async_trait;
use meditation_application::ports::{AudioFormat, ConvertError, OpusConverter, StoredAudio};
use tokio::{
    io::AsyncWriteExt,
    process::{Child, Command},
};

/// Converts accepted uploads to their stored form: compressed inputs
/// (MP3, Ogg Opus) pass through byte-identical and PCM inputs (WAV,
/// Ogg Vorbis) are compressed losslessly to FLAC.
#[derive(Clone, Copy, Default)]
pub struct FfmpegOpusConverter;

const FFMPEG: &str = "ffmpeg";
const FFPROBE: &str = "ffprobe";

fn encode_command() -> Command {
    let mut command = Command::new(FFMPEG);
    command
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-y")
        .arg("-i")
        .arg("pipe:0")
        .arg("-c:a")
        .arg("flac")
        .arg("-compression_level")
        .arg("8")
        // FLAC stream durations are only written when the muxer sees a
        // seekable output, so the encode targets a temp file and the bytes
        // are read back afterwards.
        .arg("-f")
        .arg("flac");
    command
}

async fn run_encode(mut child: Child, bytes: Vec<u8>) -> Result<(), ConvertError> {
    let mut stdin = child.stdin.take().ok_or(ConvertError::Unavailable)?;
    let input = tokio::task::spawn(async move {
        // Writing large inputs can block until ffmpeg drains them; feed it
        // from a task so stderr does not deadlock the pipeline.
        let _ = stdin.write_all(&bytes).await;
        let _ = stdin.shutdown().await;
    });

    let output = child.wait_with_output().await;
    input.await.map_err(|_| ConvertError::Unavailable)?;

    let output = output.map_err(|_| ConvertError::Unavailable)?;
    if !output.status.success() {
        // The full decode gate: a file whose magic passed but whose stream
        // breaks fails here, so nothing is stored (spec FR-004).
        return Err(ConvertError::Decode);
    }
    Ok(())
}

/// Stages `bytes` at `path` without duplicating them: `tokio::fs::write` copies
/// the whole slice into an owned buffer first, doubling the memory of a large
/// upload, while `write_all` on a file copies through a small bounded buffer.
async fn stage_bytes(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = tokio::fs::File::create(path).await?;
    file.write_all(bytes).await?;
    file.flush().await
}

/// Probes the stored bytes for their duration; a probe failure is not
/// fatal to the upload, it just leaves progress unknown (`0`). ffprobe
/// cannot size a container from a pipe (`format=duration` reports `N/A`),
/// so the bytes are staged to a temp file first. MP3 and Opus sniff their
/// extension from the magic bytes.
async fn probe_duration_ms(bytes: &[u8]) -> i64 {
    let extension = if bytes.starts_with(b"fLaC") {
        "flac"
    } else if bytes.starts_with(b"OggS") {
        "ogg"
    } else if bytes.starts_with(b"ID3") || bytes.first().is_some_and(|b| *b == 0xFF) {
        "mp3"
    } else {
        "bin"
    };
    let Ok(file) = tempfile::Builder::new()
        .suffix(&format!(".{extension}"))
        .tempfile()
    else {
        tracing::warn!("duration probe skipped: no writable temp file; duration stored as 0");
        return 0;
    };
    if let Err(error) = stage_bytes(file.path(), bytes).await {
        tracing::warn!(%error, "duration probe skipped: staging failed; duration stored as 0");
        return 0;
    }
    let output = Command::new(FFPROBE)
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg("-i")
        .arg(file.path())
        .output()
        .await;
    // Keep the temp file alive (and thus on disk) until ffprobe is done
    // reading it; dropping `file` here would delete it mid-read.
    drop(file);
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
        bytes: Vec<u8>,
        format: AudioFormat,
    ) -> Result<(Vec<u8>, i64, StoredAudio), ConvertError> {
        let stored = match format {
            AudioFormat::OggOpus => StoredAudio {
                extension: "opus",
                content_type: "audio/ogg",
            },
            AudioFormat::Mp3 => StoredAudio {
                extension: "mp3",
                content_type: "audio/mpeg",
            },
            _ => StoredAudio {
                extension: "flac",
                content_type: "audio/flac",
            },
        };
        let opus_bytes = match format {
            AudioFormat::OggOpus | AudioFormat::Mp3 => bytes,
            _ => {
                let Ok(out) = tempfile::Builder::new().suffix(".flac").tempfile() else {
                    return Err(ConvertError::Unavailable);
                };
                let out_path = out.path().to_owned();
                let mut command = encode_command();
                command.arg(&out_path);
                command.stdin(std::process::Stdio::piped());
                command.stdout(std::process::Stdio::null());
                command.stderr(std::process::Stdio::piped());
                let child = match command.spawn() {
                    Ok(child) => child,
                    // A missing encoder is an operator problem, not bad media; the
                    // API maps it to 503 later.
                    Err(_) => return Err(ConvertError::Unavailable),
                };
                run_encode(child, bytes).await?;
                match tokio::fs::read(&out_path).await {
                    Ok(bytes) if bytes.starts_with(b"fLaC") => bytes,
                    _ => return Err(ConvertError::Decode),
                }
            }
        };
        let duration_ms = probe_duration_ms(&opus_bytes).await;
        Ok((opus_bytes, duration_ms, stored))
    }
}

#[cfg(test)]
#[path = "converter.tests.rs"]
mod tests;
