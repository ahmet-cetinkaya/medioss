use super::fold;

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
