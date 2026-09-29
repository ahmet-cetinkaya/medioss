use kernel::UserId;

use crate::{ports::MoodEntryRepository, ApplicationError};

pub async fn delete_mood_entry<R>(
    entries: &R,
    user_id: UserId,
    entry_id: i64,
) -> Result<(), ApplicationError>
where
    R: MoodEntryRepository + ?Sized,
{
    if entries
        .delete(user_id, entry_id)
        .await
        .map_err(ApplicationError::Repository)?
    {
        Ok(())
    } else {
        Err(ApplicationError::NotFound)
    }
}
