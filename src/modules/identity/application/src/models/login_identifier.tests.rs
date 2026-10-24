use super::normalize_login_identifier;

#[test]
fn trims_surrounding_whitespace_and_lowercases() {
    assert_eq!(
        normalize_login_identifier("  Member@Local.Invalid \t"),
        "member@local.invalid"
    );
}

#[test]
fn already_canonical_identifier_is_unchanged() {
    assert_eq!(normalize_login_identifier("a@b.invalid"), "a@b.invalid");
}
