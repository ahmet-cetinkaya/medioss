use super::normalize_note;
use crate::ApplicationError;

#[test]
fn none_stays_none() {
    assert_eq!(normalize_note(None).unwrap(), None);
}

#[test]
fn blank_note_clears_to_none() {
    assert_eq!(normalize_note(Some("")).unwrap(), None);
    assert_eq!(normalize_note(Some("   \t\n  ")).unwrap(), None);
}

#[test]
fn note_is_trimmed() {
    assert_eq!(
        normalize_note(Some("  restless night  ")).unwrap(),
        Some("restless night".to_owned())
    );
}

#[test]
fn note_at_the_10_000_character_limit_is_kept() {
    let note = "a".repeat(10_000);
    assert_eq!(normalize_note(Some(&note)).unwrap(), Some(note));
}

#[test]
fn note_past_the_10_000_character_limit_is_rejected() {
    let note = "a".repeat(10_001);
    assert_eq!(
        normalize_note(Some(&note)),
        Err(ApplicationError::InvalidInput(
            "mood note must be at most 10000 characters"
        ))
    );
}

#[test]
fn the_limit_counts_characters_not_bytes() {
    // 6000 two-byte characters = 12000 bytes but only 6000 chars: accepted.
    let note = "é".repeat(6_000);
    assert!(normalize_note(Some(&note)).is_ok());
}
