use super::*;

#[test]
fn trims_surrounding_whitespace() {
    assert_eq!(valid_name("  Rain  "), Ok("Rain".to_owned()));
}

#[test]
fn rejects_empty_and_blank_names() {
    assert_eq!(valid_name(""), Err(DomainError::InvalidName));
    assert_eq!(valid_name("   \t "), Err(DomainError::InvalidName));
}

#[test]
fn accepts_exactly_the_maximum_length() {
    let name = "a".repeat(MAX_NAME_LENGTH);

    assert_eq!(valid_name(&name), Ok(name));
}

#[test]
fn rejects_one_character_past_the_maximum() {
    let name = "a".repeat(MAX_NAME_LENGTH + 1);

    assert_eq!(valid_name(&name), Err(DomainError::InvalidName));
}

#[test]
fn length_counts_characters_not_bytes() {
    let name = "ş".repeat(MAX_NAME_LENGTH);

    assert_eq!(valid_name(&name), Ok(name));
}
