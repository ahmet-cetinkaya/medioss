use super::matches;

#[test]
fn multi_term_is_and() {
    assert!(matches("morning breath", "Morning Breathing"));
    assert!(!matches("morning breath", "Evening Breathing"));
    assert!(matches("MORNING", "Morning Breathing"));
}

#[test]
fn blank_query_matches_everything() {
    assert!(matches("   ", "anything"));
    assert!(matches("", "anything"));
}

#[test]
fn no_match_returns_false() {
    assert!(!matches("xyzzy", "Morning Breathing"));
}
