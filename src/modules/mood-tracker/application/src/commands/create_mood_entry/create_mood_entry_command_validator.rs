use super::create_mood_entry_command::CreateMoodEntryCommand;
use crate::services::normalize_note::normalize_note;
use crate::ApplicationError;
use mood_tracker_domain::MoodScore;

/// Input rules for creating a mood entry: a score from 1 to 5 and a note of at
/// most 10000 characters.
#[derive(Debug, Default, Clone, Copy)]
pub struct CreateMoodEntryCommandValidator;

impl CreateMoodEntryCommandValidator {
    pub fn validate(&self, command: &CreateMoodEntryCommand) -> Result<(), ApplicationError> {
        self.score(command)?;
        self.normalized_note(command)?;
        Ok(())
    }

    pub fn score(&self, command: &CreateMoodEntryCommand) -> Result<MoodScore, ApplicationError> {
        MoodScore::new(command.score)
            .map_err(|_| ApplicationError::InvalidInput("mood score must be between 1 and 5"))
    }

    pub fn normalized_note(
        &self,
        command: &CreateMoodEntryCommand,
    ) -> Result<Option<String>, ApplicationError> {
        normalize_note(command.note.as_deref())
    }
}

#[cfg(test)]
#[path = "create_mood_entry_command_validator.tests.rs"]
mod tests;
