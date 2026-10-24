use crate::ports::media::ImageNormalizer;
use crate::ApplicationError;
use soundscape_domain::{sniff_icon, MAX_ICON_BYTES};

pub(crate) async fn normalize_icon(
    normalizer: &dyn ImageNormalizer,
    bytes: Vec<u8>,
) -> Result<Vec<u8>, ApplicationError> {
    if bytes.len() > MAX_ICON_BYTES {
        return Err(ApplicationError::PayloadTooLarge);
    }
    sniff_icon(&bytes).ok_or(ApplicationError::UnsupportedMedia)?;
    normalizer
        .normalize(bytes)
        .await
        .map_err(ApplicationError::from)
}

#[cfg(test)]
#[path = "normalize_icon.tests.rs"]
mod tests;
