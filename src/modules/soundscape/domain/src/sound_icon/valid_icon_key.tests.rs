use super::*;

#[test]
fn icon_key_is_normalized_and_bounded() {
    assert_eq!(valid_icon_key("  rain "), Ok(Some("rain".into())));
    assert_eq!(valid_icon_key(" "), Ok(None));
    assert!(valid_icon_key("Rain").is_err());
    assert!(valid_icon_key(&"a".repeat(65)).is_err());
}
