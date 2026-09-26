use super::*;

const STORED_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$YWJj";

#[test]
fn stored_hash_is_available_to_password_verification() {
    let password_hash = PasswordHash::from_stored(STORED_HASH.to_owned());

    assert_eq!(password_hash.as_str(), STORED_HASH);
}

#[test]
fn debug_does_not_contain_the_stored_hash() {
    let password_hash = PasswordHash::from_stored(STORED_HASH.to_owned());
    let debug = format!("{password_hash:?}");

    assert_eq!(debug, "PasswordHash([REDACTED])");
    assert!(!debug.contains(STORED_HASH));
}

#[test]
fn manual_qa_displays_a_redacted_password_hash() {
    let password_hash = PasswordHash::from_stored(STORED_HASH.to_owned());

    println!("password_hash_debug={password_hash:?}");
    assert!(!format!("{password_hash:?}").contains(password_hash.as_str()));
}
