use std::process::Stdio;

use meditation_application::ports::{AudioFormat, OpusConverter};

use super::FfmpegOpusConverter;

fn ffmpeg_available() -> bool {
    std::process::Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

async fn ffmpeg_generate(args: &[&str]) -> Vec<u8> {
    let output = tokio::process::Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .args(args)
        .output()
        .await
        .expect("ffmpeg fixture generation");
    assert!(output.status.success(), "fixture generation failed");
    output.stdout
}

#[tokio::test]
async fn wav_converts_to_lossless_flac() {
    if !ffmpeg_available() {
        eprintln!("skipping: ffmpeg not on PATH");
        return;
    }
    let wav = ffmpeg_generate(&[
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=440:duration=2",
        "-ar",
        "48000",
        "-f",
        "wav",
        "-",
    ])
    .await;
    let (flac, duration_ms, stored) = FfmpegOpusConverter
        .convert(wav.clone(), AudioFormat::Wav)
        .await
        .expect("conversion");
    assert!(flac.starts_with(b"fLaC"), "output must be a FLAC stream");
    assert_eq!(stored.extension, "flac");
    assert_eq!(stored.content_type, "audio/flac");
    assert!(
        flac.len() < wav.len(),
        "level-8 FLAC must be smaller than raw WAV: {} vs {}",
        flac.len(),
        wav.len()
    );
    assert!(
        (1_800..2_200).contains(&duration_ms),
        "a 2s fixture must probe close to 2000ms, got {duration_ms}"
    );
    // Lossless means bit-exact: decoding the FLAC must reproduce the
    // source PCM the WAV carries.
    let decoded_md5 = decode_md5(&flac).await;
    let source_md5 = decode_md5(&wav).await;
    assert_eq!(
        decoded_md5, source_md5,
        "FLAC must decode bit-exact to the WAV"
    );
}

async fn decode_md5(bytes: &[u8]) -> String {
    use tokio::io::AsyncWriteExt;
    let mut child = tokio::process::Command::new("ffmpeg")
        .args(["-i", "pipe:0", "-f", "md5", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("ffmpeg md5 decode");
    let mut stdin = child.stdin.take().expect("stdin open");
    stdin.write_all(bytes).await.expect("feed input");
    drop(stdin);
    let output = child.wait_with_output().await.expect("decode finishes");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

#[tokio::test]
async fn already_opus_passes_through_byte_identical() {
    if !ffmpeg_available() {
        eprintln!("skipping: ffmpeg not on PATH");
        return;
    }
    let opus = ffmpeg_generate(&[
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=440:duration=1",
        "-c:a",
        "libopus",
        "-f",
        "ogg",
        "-",
    ])
    .await;
    let (stored, _duration_ms, stored_format) = FfmpegOpusConverter
        .convert(opus.clone(), AudioFormat::OggOpus)
        .await
        .expect("passthrough");
    assert_eq!(stored, opus, "no re-encode, no byte drift");
    assert_eq!(stored_format.extension, "opus");
    assert_eq!(stored_format.content_type, "audio/ogg");
}

#[tokio::test]
async fn corrupt_mid_stream_input_fails_the_decode_gate() {
    if !ffmpeg_available() {
        eprintln!("skipping: ffmpeg not on PATH");
        return;
    }
    // ID3 header the sniffer accepts, followed by zero decodable frames.
    // ffmpeg exits non-zero and nothing is stored (spec FR-004's second
    // gate). Note: ffmpeg tolerates truncated or bit-flipped payloads —
    // it decodes the salvageable part — so the gate's real catch is
    // input that cannot be decoded at all.
    let mut junk = b"ID3\x04\x00\x00\x00\x00\x00\x00".to_vec();
    junk.extend(std::iter::repeat_n(0xEE, 5000));
    let result = FfmpegOpusConverter
        .convert(junk.clone(), AudioFormat::Wav)
        .await;
    assert!(result.is_err(), "an undecodable stream must not be stored");
    // MP3 uploads are stored byte-identical, so the sniffed format alone
    // must not reach storage without a decode gate: a corrupt MP3 still
    // has to fail somewhere. The upload pipeline keeps the sniffer's
    // magic gate (sync bytes) and ffprobe duration check: junk with no
    // MPEG frames probes to no duration, which upload_meditation rejects.
    let mp3_result = FfmpegOpusConverter.convert(junk, AudioFormat::Mp3).await;
    let (_, duration_ms, _) = mp3_result.expect("mp3 bytes are passed through");
    assert_eq!(
        duration_ms, 0,
        "undecodable mp3 probes no duration, so the upload is rejected downstream"
    );
}
