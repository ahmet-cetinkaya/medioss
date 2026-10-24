use super::update_mood_entry_command::UpdateMoodEntryCommand;
use crate::services::normalize_note::normalize_note;
use crate::ApplicationError;
use mood_tracker_domain::MoodScore;

/// Input rules for updating a mood entry: a score from 1 to 5 and a note of at
/// most 10000 characters.
#[derive(Debug, Default, Clone, Copy)]
pub struct UpdateMoodEntryCommandValidator;

impl UpdateMoodEntryCommandValidator {
    pub fn validate(&self, command: &UpdateMoodEntryCommand) -> Result<(), ApplicationError> {
        self.score(command)?;
        self.normalized_note(command)?;
        Ok(())
    }

    pub fn score(&self, command: &UpdateMoodEntryCommand) -> Result<MoodScore, ApplicationError> {
        MoodScore::new(command.score)
            .map_err(|_| ApplicationError::InvalidInput("mood score must be between 1 and 5"))
    }

    pub fn normalized_note(
        &self,
        command: &UpdateMoodEntryCommand,
    ) -> Result<Option<String>, ApplicationError> {
        normalize_note(command.note.as_deref())
    }
}

#[cfg(test)]
#[path = "update_mood_entry_command_validator.tests.rs"]
mod tests;
