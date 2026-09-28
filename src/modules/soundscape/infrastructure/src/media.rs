use async_trait::async_trait;
use soundscape_application::ports::media::{
    AudioFormat, AudioValidator, BlobReader, BlobStore, OpusConverter, StoredAudio,
};
use tokio::{io::AsyncWriteExt, process::Command};

pub struct FfmpegAudioValidator;
#[async_trait]
impl AudioValidator for FfmpegAudioValidator {
    async fn sniff(&self, bytes: &[u8]) -> Result<AudioFormat, ()> {
        if bytes.starts_with(b"OggS") {
            return if bytes[..bytes.len().min(64)]
                .windows(8)
                .any(|x| x == b"OpusHead")
            {
                Ok(AudioFormat::OggOpus)
            } else {
                Ok(AudioFormat::OggVorbis)
            };
        }
        if bytes.starts_with(b"RIFF") && bytes.len() >= 12 && &bytes[8..12] == b"WAVE" {
            return Ok(AudioFormat::Wav);
        }
        if bytes.starts_with(b"ID3")
            || (bytes.len() > 1 && bytes[0] == 0xff && bytes[1] & 0xe0 == 0xe0)
        {
            return Ok(AudioFormat::Mp3);
        }
        Err(())
    }
}

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

async fn probe_duration(bytes: &[u8], extension: &str) -> Result<i64, String> {
    let file = tempfile::Builder::new()
        .suffix(&format!(".{extension}"))
        .tempfile()
        .map_err(|_| "duration staging unavailable".to_owned())?;
    tokio::fs::write(file.path(), bytes)
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

/// Maps a stored blob's extension to the MIME type served with it. The store
/// keeps the original upload format, so the answer follows the file.
fn content_type_for(key: &str) -> &'static str {
    match key.rsplit('.').next().unwrap_or_default() {
        "opus" => "audio/ogg",
        "flac" => "audio/flac",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        _ => "application/octet-stream",
    }
}

pub struct MediaStoreBlobs(pub media_store::MediaStore);
#[async_trait]
impl BlobStore for MediaStoreBlobs {
    async fn put(
        &self,
        folder: &str,
        stem: &str,
        ext: &str,
        bytes: &[u8],
    ) -> Result<String, String> {
        self.0
            .put(folder, stem, ext, bytes)
            .await
            .map(|x| x.as_str().to_owned())
            .map_err(|e| e.to_string())
    }
    async fn open_range(
        &self,
        key: &str,
        start: u64,
        length: u64,
    ) -> Result<(BlobReader, String, u64), String> {
        let key = media_store::StorageKey::parse_with_separator(key).map_err(|e| e.to_string())?;
        let size = self.0.size(&key).await.map_err(|e| e.to_string())?;
        let stream = self
            .0
            .open_range(&key, start, length)
            .await
            .map_err(|e| e.to_string())?;
        let content_type = content_type_for(key.as_str());
        Ok((BlobReader(Box::new(stream)), content_type.to_owned(), size))
    }
    async fn delete(&self, key: &str) -> Result<(), String> {
        let key = media_store::StorageKey::parse_with_separator(key).map_err(|e| e.to_string())?;
        self.0.delete(&key).await.map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn wav_converts_to_lossless_flac_with_positive_duration() {
        let wav = std::process::Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=1",
                "-f",
                "wav",
                "pipe:1",
            ])
            .output()
            .expect("ffmpeg runs")
            .stdout;
        let (flac, duration, stored) = FfmpegOpusConverter
            .convert(wav.clone(), AudioFormat::Wav)
            .await
            .expect("convert wav");
        assert!(flac.starts_with(b"fLaC"));
        assert!(duration > 0);
        assert_eq!(stored.extension, "flac");
        assert_eq!(stored.content_type, "audio/flac");
        // Lossless round trip: FLAC decode must be sample-identical to the
        // source WAV decode, not merely audibly similar.
        let original_md5 = md5_of_pcm(&wav);
        let flac_md5 = md5_of_pcm(&flac);
        assert_eq!(
            original_md5, flac_md5,
            "FLAC output must decode bit-exact to the WAV input"
        );
    }

    #[tokio::test]
    async fn mp3_passes_through_byte_identical() {
        let mp3 = std::process::Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=1",
                "-b:a",
                "128k",
                "-f",
                "mp3",
                "pipe:1",
            ])
            .output()
            .expect("ffmpeg runs")
            .stdout;
        let (stored_bytes, duration, stored) = FfmpegOpusConverter
            .convert(mp3.clone(), AudioFormat::Mp3)
            .await
            .expect("convert mp3");
        assert_eq!(stored_bytes, mp3);
        assert!(duration > 0);
        assert_eq!(stored.extension, "mp3");
        assert_eq!(stored.content_type, "audio/mpeg");
    }

    fn md5_of_pcm(bytes: &[u8]) -> String {
        use std::io::Write;
        let mut child = std::process::Command::new("ffmpeg")
            .args(["-i", "pipe:0", "-f", "md5", "-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("ffmpeg runs");
        child
            .stdin
            .as_mut()
            .expect("stdin open")
            .write_all(bytes)
            .expect("write input");
        let output = child.wait_with_output().expect("ffmpeg finishes");
        String::from_utf8(output.stdout)
            .expect("md5 text")
            .trim()
            .to_owned()
    }
}
