use super::*;
use chrono::TimeZone;

fn created_at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap()
}

fn sound(name: &str, content_type: &str, storage_key: &str) -> Result<Sound, DomainError> {
    Sound::new(
        1,
        name,
        2,
        content_type,
        1_000,
        storage_key,
        5_000,
        created_at(),
    )
}

#[test]
fn builds_a_sound_with_defaults_for_optional_fields() {
    let sound = sound("  Rain ", STORED_CONTENT_TYPE, "soundscape/rain.ogg").unwrap();

    assert_eq!(sound.name, "Rain");
    assert_eq!(sound.category_id, 2);
    assert_eq!(sound.size_bytes, 1_000);
    assert_eq!(sound.duration_ms, 5_000);
    assert_eq!(sound.created_at, created_at());
    assert!(!sound.builtin);
    assert_eq!(sound.icon_key, None);
    assert_eq!(sound.icon, None);
}

#[test]
fn rejects_non_positive_media_metadata() {
    let now = created_at();
    for (category_id, size_bytes, duration_ms) in [(0, 1, 1), (1, 0, 1), (1, 1, 0), (-1, 1, 1)] {
        assert_eq!(
            Sound::new(
                1,
                "Rain",
                category_id,
                "audio/ogg",
                size_bytes,
                "k",
                duration_ms,
                now
            ),
            Err(DomainError::InvalidMedia)
        );
    }
}

#[test]
fn rejects_an_unsupported_content_type() {
    assert_eq!(
        sound("Rain", "audio/mp4", "k"),
        Err(DomainError::InvalidContentType)
    );
}

#[test]
fn rejects_an_empty_storage_key() {
    assert_eq!(
        sound("Rain", "audio/ogg", ""),
        Err(DomainError::InvalidStorageKey)
    );
}

#[test]
fn rejects_an_invalid_name() {
    assert_eq!(
        sound("   ", "audio/ogg", "k"),
        Err(DomainError::InvalidName)
    );
}
