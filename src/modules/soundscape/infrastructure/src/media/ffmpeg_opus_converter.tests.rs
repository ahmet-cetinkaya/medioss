use super::*;
use crate::media::TempFileStaging;
use soundscape_application::ports::staged_upload::UploadStaging;

async fn stage(staging: &TempFileStaging, bytes: &[u8]) -> StagedMedia {
    let mut sink = staging.open().await.unwrap();
    sink.append(bytes).await.unwrap();
    sink.keep().await.unwrap()
}

fn read_staged(root: &Path, media: &StagedMedia) -> Vec<u8> {
    std::fs::read(root.join(&media.address)).unwrap()
}

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
    let root = tempfile::tempdir().unwrap();
    let staging = TempFileStaging::new(root.path().to_path_buf());
    let converter = FfmpegOpusConverter::new(root.path().to_path_buf());
    let (converted, duration, stored) = converter
        .convert(stage(&staging, &wav).await, AudioFormat::Wav)
        .await
        .expect("convert wav");
    let flac = read_staged(root.path(), &converted);
    assert_eq!(converted.size_bytes, flac.len() as u64);
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
    let root = tempfile::tempdir().unwrap();
    let staging = TempFileStaging::new(root.path().to_path_buf());
    let converter = FfmpegOpusConverter::new(root.path().to_path_buf());
    let input = stage(&staging, &mp3).await;
    let (converted, duration, stored) = converter
        .convert(input.clone(), AudioFormat::Mp3)
        .await
        .expect("convert mp3");
    assert_eq!(converted.address, input.address);
    assert_eq!(read_staged(root.path(), &converted), mp3);
    assert!(duration > 0);
    assert_eq!(stored.extension, "mp3");
    assert_eq!(stored.content_type, "audio/mpeg");
}

#[tokio::test]
async fn undecodable_pcm_fails_and_leaves_no_converted_file() {
    let root = tempfile::tempdir().unwrap();
    let staging = TempFileStaging::new(root.path().to_path_buf());
    let converter = FfmpegOpusConverter::new(root.path().to_path_buf());
    let input = stage(&staging, b"RIFF\x04\0\0\0WAVEgarbage").await;

    let result = converter.convert(input, AudioFormat::Wav).await;

    assert!(result.is_err());
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
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
