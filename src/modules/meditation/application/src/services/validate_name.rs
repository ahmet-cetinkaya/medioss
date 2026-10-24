use crate::ApplicationError;

/// Name length cap, mirroring the domain invariant.
pub const MAX_NAME_LENGTH: usize = 200;

pub(crate) fn validate_name(name: &str) -> Result<(), ApplicationError> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.chars().count() > MAX_NAME_LENGTH {
        return Err(ApplicationError::InvalidInput(
            "name must be 1-200 characters",
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "validate_name.tests.rs"]
mod tests;
