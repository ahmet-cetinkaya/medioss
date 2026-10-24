use crate::ApplicationError;
use soundscape_domain::valid_icon_key;

pub(crate) fn parse_icon_key(icon_key: Option<&str>) -> Result<Option<String>, ApplicationError> {
    Ok(icon_key
        .map(valid_icon_key)
        .transpose()
        .map_err(|_| ApplicationError::InvalidInput("icon key"))?
        .flatten())
}

#[cfg(test)]
#[path = "parse_icon_key.tests.rs"]
mod tests;
