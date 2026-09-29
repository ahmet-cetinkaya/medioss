use kernel::UserId;
use mood_tracker_domain::{MoodEntry, MoodScore};

use crate::{
    ports::{Clock, MoodEntryRepository},
    ApplicationError,
};

pub async fn create_mood_entry<R, C>(
    entries: &R,
    clock: &C,
    user_id: UserId,
    score: u8,
    note: Option<&str>,
) -> Result<MoodEntry, ApplicationError>
where
    R: MoodEntryRepository + ?Sized,
    C: Clock + ?Sized,
{
    let score = MoodScore::new(score)
        .map_err(|_| ApplicationError::InvalidInput("mood score must be between 1 and 5"))?;
    let note = normalize_note(note)?;
    entries
        .insert(user_id, score, note, clock.now().await)
        .await
        .map_err(ApplicationError::Repository)
}

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
