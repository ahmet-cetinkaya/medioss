use std::sync::Arc;

use super::update_note_command::UpdateNoteCommand;
use super::update_note_command_validator::UpdateNoteCommandValidator;
use crate::ports::CommandHandler;
use crate::ports::{Clock, NoteRepository};
use crate::ApplicationError;

pub struct UpdateNoteCommandHandler {
    validator: UpdateNoteCommandValidator,
    notes: Arc<dyn NoteRepository>,
    clock: Arc<dyn Clock>,
}

impl UpdateNoteCommandHandler {
    pub fn new(notes: Arc<dyn NoteRepository>, clock: Arc<dyn Clock>) -> Self {
        Self {
            validator: UpdateNoteCommandValidator,
            notes,
            clock,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<UpdateNoteCommand> for UpdateNoteCommandHandler {
    type Output = ();

    async fn handle(&self, command: UpdateNoteCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;
        // Another author's, unknown, or mismatched-meditation id is NotFound
        // (spec FR-036/037).
        let UpdateNoteCommand {
            note_id,
            meditation_id,
            user_id,
            text,
        } = command;
        let text = text.trim();
        let updated = self
            .notes
            .update(
                note_id,
                meditation_id,
                user_id,
                text.to_owned(),
                self.clock.now().await,
            )
            .await
            .map_err(ApplicationError::Repository)?;
        if updated {
            Ok(())
        } else {
            Err(ApplicationError::NotFound)
        }
    }
}

#[cfg(test)]
#[path = "update_note_command_handler.tests.rs"]
mod tests;
