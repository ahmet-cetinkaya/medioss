use kernel::UserId;

use super::CreateMoodEntryCommandValidator;
use crate::commands::create_mood_entry::CreateMoodEntryCommand;
use crate::ApplicationError;

fn command(score: u8, note: Option<&str>) -> CreateMoodEntryCommand {
    CreateMoodEntryCommand {
        user_id: UserId::new(1),
        score,
        note: note.map(str::to_owned),
    }
}

#[test]
fn accepts_scores_one_to_five_with_or_without_a_note() {
    let validator = CreateMoodEntryCommandValidator;

    for score in 1..=5 {
        assert_eq!(validator.validate(&command(score, None)), Ok(()));
    }
    assert_eq!(validator.validate(&command(3, Some("fine"))), Ok(()));
}

#[test]
fn rejects_scores_outside_one_to_five() {
    let validator = CreateMoodEntryCommandValidator;

    for score in [0, 6, 255] {
        assert_eq!(
            validator.validate(&command(score, None)),
            Err(ApplicationError::InvalidInput(
                "mood score must be between 1 and 5"
            )),
            "score {score}"
        );
    }
}

#[test]
fn rejects_a_note_over_ten_thousand_characters() {
    let validator = CreateMoodEntryCommandValidator;
    let note = "x".repeat(10_001);

    assert!(matches!(
        validator.validate(&command(3, Some(&note))),
        Err(ApplicationError::InvalidInput(_))
    ));
}

#[test]
fn normalizes_a_note_by_trimming_and_dropping_blank_ones() {
    let validator = CreateMoodEntryCommandValidator;

    assert_eq!(
        validator.normalized_note(&command(3, Some("  calm  "))),
        Ok(Some("calm".to_owned()))
    );
    assert_eq!(
        validator.normalized_note(&command(3, Some("   "))),
        Ok(None)
    );
    assert_eq!(validator.normalized_note(&command(3, None)), Ok(None));
}
