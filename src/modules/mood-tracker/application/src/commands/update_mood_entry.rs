use kernel::UserId;
use mood_tracker_domain::{MoodEntry, MoodScore};

use crate::{
    commands::create_mood_entry::normalize_note,
    ports::{Clock, MoodEntryRepository},
    ApplicationError,
};

pub async fn update_mood_entry<R, C>(
    entries: &R,
    clock: &C,
    user_id: UserId,
    entry_id: i64,
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
        .update(user_id, entry_id, score, note, clock.now().await)
        .await
        .map_err(ApplicationError::Repository)?
        .ok_or(ApplicationError::NotFound)
}
