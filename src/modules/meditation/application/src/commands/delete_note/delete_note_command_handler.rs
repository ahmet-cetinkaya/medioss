use std::sync::Arc;

use super::delete_note_command::DeleteNoteCommand;
use crate::ports::CommandHandler;
use crate::ports::NoteRepository;
use crate::ApplicationError;

pub struct DeleteNoteCommandHandler {
    notes: Arc<dyn NoteRepository>,
}

impl DeleteNoteCommandHandler {
    pub fn new(notes: Arc<dyn NoteRepository>) -> Self {
        Self { notes }
    }
}

#[async_trait::async_trait]
impl CommandHandler<DeleteNoteCommand> for DeleteNoteCommandHandler {
    type Output = ();

    async fn handle(&self, command: DeleteNoteCommand) -> Result<(), ApplicationError> {
        let DeleteNoteCommand {
            note_id,
            meditation_id,
            user_id,
        } = command;
        let deleted = self
            .notes
            .delete(note_id, meditation_id, user_id)
            .await
            .map_err(ApplicationError::Repository)?;
        if deleted {
            Ok(())
        } else {
            Err(ApplicationError::NotFound)
        }
    }
}

#[cfg(test)]
#[path = "delete_note_command_handler.tests.rs"]
mod tests;
