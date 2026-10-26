use super::UploadLimits;

#[test]
fn default_limits_match_the_documented_sizes() {
    assert_eq!(
        UploadLimits::default(),
        UploadLimits {
            meditation_audio_bytes: 100 * 1024 * 1024,
            sound_audio_bytes: 100 * 1024 * 1024,
            program_cover_bytes: 5 * 1024 * 1024,
        }
    );
}
