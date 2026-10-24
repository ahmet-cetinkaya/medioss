use super::validate_name;
use crate::ApplicationError;

#[test]
fn accepts_a_plain_name() {
    assert_eq!(validate_name("Evening Wind Down"), Ok(()));
}

#[test]
fn blank_names_are_rejected() {
    assert_eq!(
        validate_name(""),
        Err(ApplicationError::InvalidInput(
            "name must be 1-200 characters"
        ))
    );
    assert_eq!(
        validate_name("   "),
        Err(ApplicationError::InvalidInput(
            "name must be 1-200 characters"
        ))
    );
}

#[test]
fn a_200_character_name_is_accepted() {
    let name = "a".repeat(200);
    assert_eq!(validate_name(&name), Ok(()));
}

#[test]
fn a_201_character_name_is_rejected() {
    let name = "a".repeat(201);
    assert_eq!(
        validate_name(&name),
        Err(ApplicationError::InvalidInput(
            "name must be 1-200 characters"
        ))
    );
}

#[test]
fn the_limit_counts_characters_not_bytes() {
    // 150 two-byte characters = 300 bytes but only 150 chars: accepted.
    let name = "é".repeat(150);
    assert_eq!(validate_name(&name), Ok(()));
}

#[test]
fn surrounding_whitespace_does_not_count_toward_the_limit() {
    let name = format!("  {}  ", "a".repeat(200));
    assert_eq!(validate_name(&name), Ok(()));
}
