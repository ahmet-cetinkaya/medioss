use super::*;

#[test]
fn a_generated_key_is_accepted() {
    let key = StorageKey::parse("01H8XG_meditation-01.opus").unwrap();

    assert_eq!(key.as_str(), "01H8XG_meditation-01.opus");
}

#[test]
fn an_empty_key_is_rejected() {
    let error = StorageKey::parse("").unwrap_err();

    assert!(matches!(error, MediaStoreError::InvalidKey { .. }));
}

#[test]
fn a_key_longer_than_the_filename_limit_is_rejected() {
    let error = StorageKey::parse("a".repeat(MAX_KEY_LEN + 1)).unwrap_err();

    assert!(matches!(error, MediaStoreError::InvalidKey { .. }));
}

#[test]
fn a_key_at_the_filename_limit_is_accepted() {
    let key = StorageKey::parse("a".repeat(MAX_KEY_LEN)).unwrap();

    assert_eq!(key.as_str().len(), MAX_KEY_LEN);
}

#[test]
fn a_parent_directory_segment_is_rejected() {
    let error = StorageKey::parse("..").unwrap_err();

    assert!(matches!(error, MediaStoreError::InvalidKey { .. }));
}

#[test]
fn a_path_separator_is_rejected() {
    let error = StorageKey::parse("nested/key").unwrap_err();

    assert!(matches!(error, MediaStoreError::InvalidKey { .. }));
}

#[test]
fn the_rejected_key_is_named_in_the_error() {
    let error = StorageKey::parse("../etc/passwd").unwrap_err();

    assert!(error.to_string().contains("../etc/passwd"));
}
