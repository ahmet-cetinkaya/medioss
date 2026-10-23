use crate::error::DomainError;
use crate::mix::MixItem;
use crate::name::valid_name;
use chrono::{DateTime, Utc};

/// Maximum number of distinct sounds one mix may contain. Deliberately
/// generous: layering many ambient sounds is a supported use, and the only
/// ceiling is a sanity bound against runaway payloads, not a product limit.
pub const MAX_MIX_SOUNDS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mix {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub items: Vec<MixItem>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl Mix {
    pub fn new(
        id: i64,
        user_id: i64,
        name: &str,
        items: Vec<MixItem>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if user_id <= 0 {
            return Err(DomainError::InvalidMixItems);
        }
        if items.len() > MAX_MIX_SOUNDS {
            return Err(DomainError::InvalidMixItems);
        }
        let mut ids = std::collections::HashSet::new();
        for (index, item) in items.iter().enumerate() {
            if item.sound_id <= 0 || !ids.insert(item.sound_id) {
                return Err(DomainError::InvalidMixItems);
            }
            if !(0..=100).contains(&item.volume) {
                return Err(DomainError::InvalidVolume);
            }
            if item.position != index as i64 {
                return Err(DomainError::InvalidPosition);
            }
        }
        Ok(Self {
            id,
            user_id,
            name: valid_name(name)?,
            items,
            created_at,
            updated_at,
        })
    }
}

#[cfg(test)]
#[path = "mix.tests.rs"]
mod tests;
