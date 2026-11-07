use super::Locale;

#[test]
fn turkish_tags_select_turkish() {
    assert_eq!(Locale::from_tag(Some("tr")), Locale::Tr);
    assert_eq!(Locale::from_tag(Some("TR")), Locale::Tr);
    assert_eq!(Locale::from_tag(Some("tr-TR")), Locale::Tr);
}

#[test]
fn missing_or_unknown_tags_fall_back_to_english() {
    assert_eq!(Locale::from_tag(None), Locale::En);
    assert_eq!(Locale::from_tag(Some("")), Locale::En);
    assert_eq!(Locale::from_tag(Some("de")), Locale::En);
    assert_eq!(Locale::from_tag(Some("en-US")), Locale::En);
}

#[test]
fn pick_prefers_the_turkish_title_only_for_turkish() {
    assert_eq!(Locale::Tr.pick("Rain", Some("Yağmur")), "Yağmur");
    assert_eq!(Locale::En.pick("Rain", Some("Yağmur")), "Rain");
}

#[test]
fn pick_falls_back_to_the_stored_name() {
    assert_eq!(Locale::Tr.pick("My sound", None), "My sound");
    assert_eq!(Locale::Tr.pick("My sound", Some("  ")), "My sound");
}
