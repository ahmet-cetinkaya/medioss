use kernel::UserId;

use super::{CreateNoteCommand, CreateNoteCommandValidator};
use crate::ApplicationError;

fn command(text: &str) -> CreateNoteCommand {
    CreateNoteCommand {
        user_id: UserId::new(1),
        meditation_id: 1,
        text: text.into(),
    }
}

#[test]
fn accepts_text() {
    assert_eq!(
        CreateNoteCommandValidator.validate(&command("keep calm")),
        Ok(())
    );
}

#[test]
fn rejects_blank_text() {
    assert_eq!(
        CreateNoteCommandValidator.validate(&command("  \n")),
        Err(ApplicationError::InvalidInput(
            "note text must not be empty"
        ))
    );
}
