//! ffmpeg subprocess Opus conversion. The command line is fixed argv —
//! media bytes flow through pipes, never through arguments.
use async_trait::async_trait;
use meditation_application::ports::{AudioFormat, OpusConverter};
use tokio::{
    io::AsyncWriteExt,
    process::{Child, Command},
};

/// Converts uploads to Ogg Opus via `ffmpeg`; passes Opus input through
/// byte-identical.
#[derive(Clone, Copy, Default)]
pub struct FfmpegOpusConverter;

const FFMPEG: &str = "ffmpeg";

fn encode_command() -> Command {
    let mut command = Command::new(FFMPEG);
    command
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-i")
        .arg("pipe:0")
        .arg("-c:a")
        .arg("libopus")
        .arg("-b:a")
        .arg("128k")
        .arg("-vbr")
        .arg("on")
        .arg("-f")
        .arg("ogg")
        .arg("pipe:1");
    command
}

async fn run_encode(mut child: Child, bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "ffmpeg stdin was not piped".to_owned())?;
    let input = tokio::task::spawn(async move {
        // Writing large inputs can block until ffmpeg drains them; feed it
        // from a task so stderr does not deadlock the pipeline.
        let _ = stdin.write_all(&bytes).await;
        let _ = stdin.shutdown().await;
    });

    let output = child.wait_with_output().await;
    input.await.map_err(|e| format!("stdin task failed: {e}"))?;

    let output = output.map_err(|e| format!("ffmpeg failed: {e}"))?;
    if !output.status.success() {
        // The full decode gate: a file whose magic passed but whose stream
        // breaks fails here, so nothing is stored (spec FR-004).
        return Err(format!(
            "ffmpeg exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout.is_empty() || !output.stdout.starts_with(b"OggS") {
        return Err("ffmpeg produced no Ogg output".to_owned());
    }
    Ok(output.stdout)
}

#[async_trait]
impl OpusConverter for FfmpegOpusConverter {
    async fn convert(&self, bytes: Vec<u8>, format: AudioFormat) -> Result<Vec<u8>, String> {
        if format == AudioFormat::OggOpus {
            return Ok(bytes);
        }
        let mut command = encode_command();
        command.stdin(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        let child = match command.spawn() {
            Ok(child) => child,
            // A missing encoder is an operator problem, not bad media; the
            // API maps it to 503 later.
            Err(error) => return Err(format!("encoder unavailable: {error}")),
        };
        run_encode(child, bytes).await
    }
}

#[cfg(test)]
#[path = "converter.tests.rs"]
mod tests;
