use super::SessionSecret;

#[test]
fn expose_returns_the_value_the_constructor_was_given() {
    let secret = SessionSecret::new_for_tests("s3cr3t value");

    assert_eq!(secret.expose(), "s3cr3t value");
}

#[test]
fn equality_compares_the_secret_value() {
    let first = SessionSecret::new_for_tests("alpha");
    let second = SessionSecret::new_for_tests("alpha");
    let other = SessionSecret::new_for_tests("beta");

    assert_eq!(first, second);
    assert_ne!(first, other);
}

#[test]
fn debug_output_never_contains_the_secret() {
    let secret = SessionSecret::new_for_tests("hidden-value");

    assert_eq!(format!("{secret:?}"), "SessionSecret([REDACTED])");
}
