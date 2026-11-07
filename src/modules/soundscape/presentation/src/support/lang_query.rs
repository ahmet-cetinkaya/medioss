use serde::Deserialize;
use soundscape_domain::Locale;

/// `?lang=tr|en` on the list routes: the interface language the caller
/// wants built-in titles in. Unknown or missing means English.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct LangQuery {
    lang: Option<String>,
}

impl LangQuery {
    pub(crate) fn locale(&self) -> Locale {
        Locale::from_tag(self.lang.as_deref())
    }
}

#[cfg(test)]
#[path = "lang_query.tests.rs"]
mod tests;
