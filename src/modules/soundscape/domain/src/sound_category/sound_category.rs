use crate::error::DomainError;
use crate::locale::Locale;
use crate::name::valid_name;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundCategory {
    pub id: i64,
    pub name: String,
    /// Turkish title of a built-in category; `None` for user categories and
    /// renamed ones. Never written back into `name`.
    pub name_tr: Option<String>,
    pub slug: String,
}
impl SoundCategory {
    pub fn new(id: i64, name: &str, slug: &str) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            name: valid_name(name)?,
            name_tr: None,
            slug: valid_name(slug)?,
        })
    }

    /// The name to show for `locale`: the Turkish title when it exists,
    /// otherwise the stored name.
    pub fn title(&self, locale: Locale) -> &str {
        locale.pick(&self.name, self.name_tr.as_deref())
    }
}

#[cfg(test)]
#[path = "sound_category.tests.rs"]
mod tests;
