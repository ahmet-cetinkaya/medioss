use super::*;

#[test]
fn absent_or_blank_key_means_no_key() {
    assert_eq!(parse_icon_key(None), Ok(None));
    assert_eq!(parse_icon_key(Some("   ")), Ok(None));
}

#[test]
fn valid_key_is_trimmed_and_kept() {
    assert_eq!(
        parse_icon_key(Some(" cloud-rain ")),
        Ok(Some("cloud-rain".into()))
    );
}

#[test]
fn invalid_characters_are_rejected() {
    assert_eq!(
        parse_icon_key(Some("Cloud Rain")),
        Err(ApplicationError::InvalidInput("icon key"))
    );
}

#[test]
fn overlong_key_is_rejected() {
    let key = "a".repeat(soundscape_domain::MAX_ICON_KEY_LENGTH + 1);
    assert_eq!(
        parse_icon_key(Some(&key)),
        Err(ApplicationError::InvalidInput("icon key"))
    );
}
