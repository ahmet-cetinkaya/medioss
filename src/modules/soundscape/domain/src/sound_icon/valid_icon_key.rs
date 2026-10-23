use crate::error::DomainError;

pub const MAX_ICON_KEY_LENGTH: usize = 64;

/// Normalizes a bundled-glyph key: lowercase `[a-z0-9-]`, at most
/// `MAX_ICON_KEY_LENGTH`. Blank input means "no key".
pub fn valid_icon_key(key: &str) -> Result<Option<String>, DomainError> {
    let key = key.trim();
    if key.is_empty() {
        return Ok(None);
    }
    let ok = key.len() <= MAX_ICON_KEY_LENGTH
        && key
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if ok {
        Ok(Some(key.to_owned()))
    } else {
        Err(DomainError::InvalidIconKey)
    }
}

#[cfg(test)]
#[path = "valid_icon_key.tests.rs"]
mod tests;
