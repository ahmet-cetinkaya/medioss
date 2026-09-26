use super::*;

#[test]
fn parse_accepts_eight_characters() {
    let password = RawPassword::parse("12345678").unwrap();

    assert_eq!(password.as_str(), "12345678");
}

#[test]
fn parse_rejects_fewer_than_eight_characters() {
    assert_eq!(
        RawPassword::parse("1234567"),
        Err(IdentityError::WeakPassword)
    );
}

#[test]
fn debug_does_not_contain_the_raw_password() {
    let password = RawPassword::parse("a-secret-password").unwrap();

    assert_eq!(format!("{password:?}"), "RawPassword([REDACTED])");
}

#[test]
fn manual_qa_displays_a_redacted_raw_password() {
    let password = RawPassword::parse("a-secret-password").unwrap();

    println!("raw_password_debug={password:?}");
    assert!(!format!("{password:?}").contains(password.as_str()));
}
