use super::default_icon_key;

#[test]
fn name_words_win_over_category() {
    assert_eq!(default_icon_key("rain", "Thunder"), Some("thunder"));
    assert_eq!(
        default_icon_key("transport", "Inside A Train"),
        Some("city")
    );
    assert_eq!(default_icon_key("nature", "Wind In Trees"), Some("wind"));
    assert_eq!(
        default_icon_key("things", "Windshield Wipers"),
        Some("music")
    );
}

#[test]
fn category_is_the_fallback_and_unknown_is_none() {
    assert_eq!(default_icon_key("animals", "Owl"), Some("birds"));
    assert_eq!(default_icon_key("custom", "Thing"), None);
}
