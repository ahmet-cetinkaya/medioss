use super::FfmpegAudioValidator;
use soundscape_application::ports::media::{AudioFormat, AudioValidator};

async fn sniff(bytes: &[u8]) -> Result<AudioFormat, ()> {
    FfmpegAudioValidator.sniff(bytes).await
}

#[tokio::test]
async fn ogg_with_opus_head_is_ogg_opus() {
    let mut bytes = b"OggS".to_vec();
    bytes.extend_from_slice(&[0; 20]);
    bytes.extend_from_slice(b"OpusHead");
    assert_eq!(sniff(&bytes).await, Ok(AudioFormat::OggOpus));
}

#[tokio::test]
async fn ogg_without_opus_head_is_ogg_vorbis() {
    assert_eq!(sniff(b"OggS\0\x02vorbis").await, Ok(AudioFormat::OggVorbis));
}

#[tokio::test]
async fn opus_head_beyond_the_first_64_bytes_is_not_recognised() {
    let mut bytes = b"OggS".to_vec();
    bytes.extend_from_slice(&[0; 80]);
    bytes.extend_from_slice(b"OpusHead");
    assert_eq!(sniff(&bytes).await, Ok(AudioFormat::OggVorbis));
}

#[tokio::test]
async fn riff_wave_is_wav() {
    assert_eq!(sniff(b"RIFF\0\0\0\0WAVEfmt ").await, Ok(AudioFormat::Wav));
}

#[tokio::test]
async fn riff_that_is_not_wave_or_too_short_is_rejected() {
    assert_eq!(sniff(b"RIFF\0\0\0\0AVI LIST").await, Err(()));
    assert_eq!(sniff(b"RIFF\0\0\0\0WAV").await, Err(()));
}

#[tokio::test]
async fn id3_tag_and_mpeg_frame_sync_are_mp3() {
    assert_eq!(sniff(b"ID3\x04\0\0").await, Ok(AudioFormat::Mp3));
    assert_eq!(sniff(&[0xff, 0xfb, 0x90, 0x00]).await, Ok(AudioFormat::Mp3));
}

#[tokio::test]
async fn unknown_empty_and_single_sync_byte_inputs_are_rejected() {
    assert_eq!(sniff(b"").await, Err(()));
    assert_eq!(sniff(&[0xff]).await, Err(()));
    assert_eq!(sniff(&[0xff, 0x00]).await, Err(()));
    assert_eq!(sniff(b"%PDF-1.7").await, Err(()));
}
