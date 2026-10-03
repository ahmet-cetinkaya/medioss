use super::*;

#[test]
fn folds_accented_letters_to_ascii() {
    assert_eq!(fold("İzmir"), "izmir");
    assert_eq!(fold("Crème Brûlée"), "creme brulee");
    assert_eq!(fold("Über Straße"), "uber straße");
}

#[test]
fn folds_cyrillic_and_greek() {
    assert_eq!(fold("Привет"), "привет");
    assert_eq!(fold("ΣΟΦΟΣ"), "σοφοσ");
}

#[test]
fn combining_marks_are_stripped() {
    assert!(!fold("i̇").contains('̇'));
}

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
