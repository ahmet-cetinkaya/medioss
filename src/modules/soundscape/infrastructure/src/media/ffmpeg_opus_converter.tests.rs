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
