use kernel::UserId;

use super::{UpdateNoteCommand, UpdateNoteCommandValidator};
use crate::ApplicationError;

fn command(text: &str) -> UpdateNoteCommand {
    UpdateNoteCommand {
        note_id: 1,
        meditation_id: 1,
        user_id: UserId::new(1),
        text: text.into(),
    }
}

#[test]
fn accepts_text() {
    assert_eq!(
        UpdateNoteCommandValidator.validate(&command("keep calm")),
        Ok(())
    );
}

#[test]
fn rejects_blank_text() {
    assert_eq!(
        UpdateNoteCommandValidator.validate(&command("  \n")),
        Err(ApplicationError::InvalidInput(
            "note text must not be empty"
        ))
    );
}
