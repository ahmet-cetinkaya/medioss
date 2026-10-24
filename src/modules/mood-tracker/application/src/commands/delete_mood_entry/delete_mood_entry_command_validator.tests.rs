use kernel::UserId;

use super::DeleteMoodEntryCommandValidator;
use crate::commands::delete_mood_entry::DeleteMoodEntryCommand;

#[test]
fn accepts_any_delete_command_because_the_repository_decides_ownership() {
    let command = DeleteMoodEntryCommand {
        user_id: UserId::new(1),
        entry_id: 1,
    };

    assert_eq!(DeleteMoodEntryCommandValidator.validate(&command), Ok(()));
}
