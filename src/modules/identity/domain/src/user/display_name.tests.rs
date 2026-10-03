use super::DisplayName;
use crate::IdentityError;

#[test]
fn accepts_trimmed_names_within_limit() {
    let name = DisplayName::parse("  Ahmet Çetinkaya  ").unwrap();
    assert_eq!(name.as_str(), "Ahmet Çetinkaya");
}

#[test]
fn counts_scalars_not_bytes() {
    let sixty_four = "ç".repeat(64);
    assert!(DisplayName::parse(sixty_four).is_ok());

    let sixty_five = "ç".repeat(65);
    assert_eq!(
        DisplayName::parse(sixty_five).unwrap_err(),
        IdentityError::InvalidProfile
    );
}

#[test]
fn rejects_empty_and_whitespace_only() {
    assert_eq!(
        DisplayName::parse("").unwrap_err(),
        IdentityError::InvalidProfile
    );
    assert_eq!(
        DisplayName::parse("   \t\n").unwrap_err(),
        IdentityError::InvalidProfile
    );
}

#[test]
fn accepts_sixty_four_ascii_chars() {
    let name = DisplayName::parse("a".repeat(64)).unwrap();
    assert_eq!(name.as_str().len(), 64);
}
