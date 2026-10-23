use crate::ApplicationError;

/// Normalizes an optional description (blank clears it) and enforces the
/// domain length limit.
pub(crate) fn validate_description(
    description: Option<&str>,
) -> Result<Option<String>, ApplicationError> {
    let text = description.map(str::trim).filter(|text| !text.is_empty());
    if text.is_some_and(|text| text.chars().count() > meditation_domain::MAX_DESCRIPTION_LENGTH) {
        return Err(ApplicationError::InvalidDescription);
    }
    Ok(text.map(str::to_owned))
}
