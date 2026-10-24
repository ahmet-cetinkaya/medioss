use std::sync::Arc;

use super::create_note_command::CreateNoteCommand;
use super::create_note_command_validator::CreateNoteCommandValidator;
use crate::ports::CommandHandler;
use crate::ports::Note;
use crate::ports::{Clock, MeditationRepository, NoteRepository};
use crate::services::ensure_meditation_exists;
use crate::ApplicationError;

pub struct CreateNoteCommandHandler {
    validator: CreateNoteCommandValidator,
    meditations: Arc<dyn MeditationRepository>,
    notes: Arc<dyn NoteRepository>,
    clock: Arc<dyn Clock>,
}

impl CreateNoteCommandHandler {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        notes: Arc<dyn NoteRepository>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            validator: CreateNoteCommandValidator,
            meditations,
            notes,
            clock,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<CreateNoteCommand> for CreateNoteCommandHandler {
    type Output = Note;

    async fn handle(&self, command: CreateNoteCommand) -> Result<Note, ApplicationError> {
        self.validator.validate(&command)?;
        let CreateNoteCommand {
            user_id,
            meditation_id,
            text,
        } = command;
        ensure_meditation_exists(self.meditations.as_ref(), meditation_id).await?;
        let text = text.trim();
        let now = self.clock.now().await;
        self.notes
            .insert(user_id, meditation_id, text.to_owned(), now)
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "create_note_command_handler.tests.rs"]
mod tests;
