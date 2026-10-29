use super::*;
use crate::meditation::STORED_CONTENT_TYPE;
use chrono::{TimeZone, Utc};

const ID: i64 = 1;
const KEY: &str = "meditation/abc.ogg";

fn created_at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 26, 8, 0, 0).unwrap()
}

fn meditation(
    name: &str,
    content_type: &str,
    size_bytes: i64,
    storage_key: &str,
) -> Result<Meditation, MeditationError> {
    Meditation::new(
        ID,
        name,
        content_type,
        size_bytes,
        storage_key,
        0,
        created_at(),
    )
}

fn valid() -> Result<Meditation, MeditationError> {
    meditation("Morning calm", STORED_CONTENT_TYPE, 42, KEY)
}

#[test]
fn happy_path_stores_and_exposes_all_fields() {
    let meditation = valid().unwrap();
    assert_eq!(meditation.id(), ID);
    assert_eq!(meditation.name(), "Morning calm");
    assert_eq!(meditation.content_type(), "audio/ogg");
    assert_eq!(meditation.size_bytes(), 42);
    assert_eq!(meditation.storage_key(), KEY);
    assert_eq!(meditation.created_at(), created_at());
}

#[test]
fn name_is_trimmed_and_blank_is_invalid() {
    assert_eq!(
        meditation("  Morning  ", STORED_CONTENT_TYPE, 42, KEY).map(|m| m.name().to_owned()),
        Ok("Morning".to_owned())
    );
    for name in ["", "   ", "\t\n"] {
        assert_eq!(
            meditation(name, STORED_CONTENT_TYPE, 42, KEY),
            Err(MeditationError::InvalidName)
        );
    }
}

#[test]
fn name_longer_than_200_characters_is_invalid() {
    let exactly_200 = "x".repeat(200);
    assert!(meditation(&exactly_200, STORED_CONTENT_TYPE, 42, KEY).is_ok());
    let too_long = "x".repeat(201);
    assert_eq!(
        meditation(&too_long, STORED_CONTENT_TYPE, 42, KEY),
        Err(MeditationError::InvalidName)
    );
}

#[test]
fn only_supported_audio_content_types_are_accepted() {
    for good in ["audio/ogg", "audio/mpeg", "audio/flac", "audio/wav"] {
        assert!(
            meditation("Morning", good, 42, KEY).is_ok(),
            "{good} must be accepted"
        );
    }
    assert_eq!(
        meditation("Morning", "audio/aac", 42, KEY),
        Err(MeditationError::InvalidContentType)
    );
    assert_eq!(
        meditation("Morning", "", 42, KEY),
        Err(MeditationError::InvalidContentType)
    );
}

#[test]
fn size_must_be_positive() {
    for size in [0, -1] {
        assert_eq!(
            meditation("Morning", STORED_CONTENT_TYPE, size, KEY),
            Err(MeditationError::InvalidSize)
        );
    }
}

#[test]
fn storage_key_must_be_non_empty() {
    assert_eq!(
        meditation("Morning", STORED_CONTENT_TYPE, 42, ""),
        Err(MeditationError::InvalidStorageKey)
    );
}

#[test]
fn description_is_trimmed_cleared_when_blank_and_length_limited() {
    let base = valid().unwrap();
    assert_eq!(base.description(), None);
    let described = base.clone().with_description(Some("  calm  ")).unwrap();
    assert_eq!(described.description(), Some("calm"));
    assert_eq!(
        described
            .with_description(Some("  "))
            .unwrap()
            .description(),
        None
    );
    let at_limit = "a".repeat(2000);
    assert!(base.clone().with_description(Some(&at_limit)).is_ok());
    assert_eq!(
        base.with_description(Some(&"a".repeat(2001))),
        Err(MeditationError::InvalidDescription)
    );
}

#[test]
fn with_name_trims_and_replaces_the_name() {
    let renamed = valid().unwrap().with_name("  Evening calm ").unwrap();
    assert_eq!(renamed.name(), "Evening calm");
}

#[test]
fn with_name_rejects_blank_and_overlong_names() {
    let blank = valid().unwrap().with_name("   ");
    assert!(matches!(blank, Err(MeditationError::InvalidName)));
    let too_long = valid().unwrap().with_name(&"a".repeat(MAX_NAME_LENGTH + 1));
    assert!(matches!(too_long, Err(MeditationError::InvalidName)));
}
