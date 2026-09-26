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
async fn wav_converts_to_smaller_ogg_opus() {
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
    let opus = FfmpegOpusConverter
        .convert(wav.clone(), AudioFormat::Wav)
        .await
        .expect("conversion");
    assert!(opus.starts_with(b"OggS"), "output must be an Ogg stream");
    assert!(
        opus.len() < wav.len(),
        "128k VBR Opus must be smaller than raw WAV: {} vs {}",
        opus.len(),
        wav.len()
    );
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
    let stored = FfmpegOpusConverter
        .convert(opus.clone(), AudioFormat::OggOpus)
        .await
        .expect("passthrough");
    assert_eq!(stored, opus, "no re-encode, no byte drift");
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
    let result = FfmpegOpusConverter.convert(junk, AudioFormat::Mp3).await;
    assert!(result.is_err(), "an undecodable stream must not be stored");
}
