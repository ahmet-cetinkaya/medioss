use crate::error::DomainError;

pub const MAX_NAME_LENGTH: usize = 80;

pub fn valid_name(name: &str) -> Result<String, DomainError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME_LENGTH {
        return Err(DomainError::InvalidName);
    }
    Ok(name.to_owned())
}
