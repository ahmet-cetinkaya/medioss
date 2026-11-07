/// The interface language a built-in title is shown in. Stored names stay
/// English; a title is only a read-time view over them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Locale {
    #[default]
    En,
    Tr,
}

impl Locale {
    /// Reads a `lang` value such as `tr` or `tr-TR`; anything unknown or
    /// missing means English.
    pub fn from_tag(tag: Option<&str>) -> Self {
        let primary = tag
            .and_then(|value| value.split(['-', '_']).next())
            .map(|value| value.trim().to_ascii_lowercase());
        match primary.as_deref() {
            Some("tr") => Self::Tr,
            _ => Self::En,
        }
    }

    /// The title to show: the Turkish one when asked for Turkish and it is
    /// not blank, otherwise the stored `name`.
    pub fn pick<'a>(self, name: &'a str, name_tr: Option<&'a str>) -> &'a str {
        match (self, name_tr) {
            (Self::Tr, Some(title)) if !title.trim().is_empty() => title,
            _ => name,
        }
    }
}

#[cfg(test)]
#[path = "locale.tests.rs"]
mod tests;
