use super::*;

#[test]
fn accepts_each_supported_audio_type() {
    for content_type in ["audio/ogg", "audio/mpeg", "audio/flac", "audio/wav"] {
        assert!(valid_content_type(content_type), "{content_type}");
    }
}

#[test]
fn rejects_other_types_and_near_misses() {
    for content_type in ["", "audio/mp4", "AUDIO/OGG", "audio/ogg ", "image/png"] {
        assert!(!valid_content_type(content_type), "{content_type:?}");
    }
}
