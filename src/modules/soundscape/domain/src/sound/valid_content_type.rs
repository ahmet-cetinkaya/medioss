/// Content types a persisted sound may carry. Compressed uploads keep their
/// original format; PCM-derived ones are stored losslessly as FLAC.
pub fn valid_content_type(content_type: &str) -> bool {
    matches!(
        content_type,
        "audio/ogg" | "audio/mpeg" | "audio/flac" | "audio/wav"
    )
}

#[cfg(test)]
#[path = "valid_content_type.tests.rs"]
mod tests;
