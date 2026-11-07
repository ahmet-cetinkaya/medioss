use super::LangQuery;
use soundscape_domain::Locale;

fn query(lang: Option<&str>) -> LangQuery {
    LangQuery {
        lang: lang.map(str::to_owned),
    }
}

#[test]
fn turkish_is_read_from_the_lang_value() {
    assert_eq!(query(Some("tr")).locale(), Locale::Tr);
    assert_eq!(query(Some("tr-TR")).locale(), Locale::Tr);
}

#[test]
fn missing_or_unknown_lang_means_english() {
    assert_eq!(query(None).locale(), Locale::En);
    assert_eq!(query(Some("xx")).locale(), Locale::En);
}
