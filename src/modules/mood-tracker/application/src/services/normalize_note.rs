use crate::ApplicationError;

pub(crate) fn normalize_note(note: Option<&str>) -> Result<Option<String>, ApplicationError> {
    let Some(note) = note else {
        return Ok(None);
    };
    let note = note.trim();
    if note.is_empty() {
        return Ok(None);
    }
    if note.chars().count() > 10_000 {
        return Err(ApplicationError::InvalidInput(
            "mood note must be at most 10000 characters",
        ));
    }
    Ok(Some(note.to_owned()))
}
