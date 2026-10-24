use super::validate_description;
use crate::ApplicationError;
use meditation_domain::MAX_DESCRIPTION_LENGTH;

#[test]
fn none_stays_none() {
    assert_eq!(validate_description(None).unwrap(), None);
}

#[test]
fn blank_description_clears_to_none() {
    assert_eq!(validate_description(Some("")).unwrap(), None);
    assert_eq!(validate_description(Some("  \n\t ")).unwrap(), None);
}

#[test]
fn description_is_trimmed() {
    assert_eq!(
        validate_description(Some("  slow breathing  ")).unwrap(),
        Some("slow breathing".to_owned())
    );
}

#[test]
fn a_description_at_max_description_length_is_accepted() {
    let description = "a".repeat(MAX_DESCRIPTION_LENGTH);
    assert_eq!(
        validate_description(Some(&description)).unwrap(),
        Some(description)
    );
}

#[test]
fn a_description_past_max_description_length_is_rejected() {
    let description = "a".repeat(MAX_DESCRIPTION_LENGTH + 1);
    assert_eq!(
        validate_description(Some(&description)),
        Err(ApplicationError::InvalidDescription)
    );
}

#[test]
fn the_limit_counts_characters_not_bytes() {
    // 1500 two-byte characters = 3000 bytes but under the 2000-char cap: accepted.
    let description = "é".repeat(1500);
    assert!(validate_description(Some(&description)).is_ok());
}
