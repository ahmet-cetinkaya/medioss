use super::*;

#[tokio::test]
async fn id3_tagged_mp3_is_detected() {
    let format = FfmpegAudioValidator
        .sniff(b"ID3\x04\x00\x00\x00\x00\x00\x00")
        .await
        .unwrap();
    assert_eq!(format, AudioFormat::Mp3);
}

#[tokio::test]
async fn bare_mpeg_sync_is_detected() {
    let mut bytes = vec![0xFF, 0xFB];
    bytes.extend_from_slice(&[0x00; 16]);
    let format = FfmpegAudioValidator.sniff(&bytes).await.unwrap();
    assert_eq!(format, AudioFormat::Mp3);
}

#[tokio::test]
async fn riff_wave_is_detected() {
    let mut bytes = b"RIFF".to_vec();
    bytes.extend_from_slice(&[0u8; 4]);
    bytes.extend_from_slice(b"WAVE");
    let format = FfmpegAudioValidator.sniff(&bytes).await.unwrap();
    assert_eq!(format, AudioFormat::Wav);
}

#[tokio::test]
async fn ogg_opus_and_vorbis_are_distinguished_by_opushead() {
    let mut opus = b"OggS".to_vec();
    opus.extend_from_slice(&[0x00; 24]);
    opus.extend_from_slice(b"OpusHead");
    let format = FfmpegAudioValidator.sniff(&opus).await.unwrap();
    assert_eq!(format, AudioFormat::OggOpus);

    let mut vorbis = b"OggS".to_vec();
    vorbis.extend_from_slice(&[0x00; 40]);
    let format = FfmpegAudioValidator.sniff(&vorbis).await.unwrap();
    assert_eq!(format, AudioFormat::OggVorbis);
}

#[tokio::test]
async fn garbage_and_short_inputs_are_rejected() {
    assert_eq!(
        FfmpegAudioValidator.sniff(b"not audio!").await.unwrap_err(),
        SniffError::Unsupported
    );
    assert_eq!(
        FfmpegAudioValidator.sniff(b"Og").await.unwrap_err(),
        SniffError::TooShort
    );
}
