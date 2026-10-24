use super::{UpdateMeditationDescriptionCommand, UpdateMeditationDescriptionCommandValidator};
use crate::ApplicationError;

fn command_with(description: Option<String>) -> UpdateMeditationDescriptionCommand {
    UpdateMeditationDescriptionCommand {
        meditation_id: 1,
        description,
    }
}

#[test]
fn accepts_a_description_within_the_limit() {
    let command = command_with(Some("calm".to_owned()));

    assert!(UpdateMeditationDescriptionCommandValidator
        .validate(&command)
        .is_ok());
}

#[test]
fn trims_the_description() {
    let command = command_with(Some("  calm  ".to_owned()));

    assert_eq!(
        UpdateMeditationDescriptionCommandValidator.normalized_description(&command),
        Ok(Some("calm".to_owned()))
    );
}

#[test]
fn blank_or_missing_description_normalizes_to_none() {
    for description in [None, Some("   ".to_owned())] {
        let command = command_with(description);

        assert_eq!(
            UpdateMeditationDescriptionCommandValidator.normalized_description(&command),
            Ok(None)
        );
    }
}

#[test]
fn rejects_a_description_over_the_limit() {
    let command = command_with(Some(
        "x".repeat(meditation_domain::MAX_DESCRIPTION_LENGTH + 1),
    ));

    assert_eq!(
        UpdateMeditationDescriptionCommandValidator.validate(&command),
        Err(ApplicationError::InvalidDescription)
    );
}
