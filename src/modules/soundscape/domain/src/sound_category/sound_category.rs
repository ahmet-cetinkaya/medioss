use crate::error::DomainError;
use crate::name::valid_name;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundCategory {
    pub id: i64,
    pub name: String,
    pub slug: String,
}
impl SoundCategory {
    pub fn new(id: i64, name: &str, slug: &str) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            name: valid_name(name)?,
            slug: valid_name(slug)?,
        })
    }
}

#[cfg(test)]
#[path = "sound_category.tests.rs"]
mod tests;
