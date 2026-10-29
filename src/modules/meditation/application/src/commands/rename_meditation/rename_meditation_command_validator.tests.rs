use super::{RenameMeditationCommand, RenameMeditationCommandValidator};
use crate::services::MAX_NAME_LENGTH;
use crate::ApplicationError;

fn rename(name: &str) -> RenameMeditationCommand {
    RenameMeditationCommand {
        meditation_id: 1,
        name: name.to_owned(),
    }
}

#[test]
fn a_plain_name_is_valid() {
    assert!(RenameMeditationCommandValidator
        .validate(&rename("Evening calm"))
        .is_ok());
}

#[test]
fn a_blank_name_is_rejected() {
    assert!(matches!(
        RenameMeditationCommandValidator.validate(&rename("   ")),
        Err(ApplicationError::InvalidInput(_))
    ));
}

#[test]
fn a_name_over_the_cap_is_rejected() {
    let too_long = "a".repeat(MAX_NAME_LENGTH + 1);
    assert!(matches!(
        RenameMeditationCommandValidator.validate(&rename(&too_long)),
        Err(ApplicationError::InvalidInput(_))
    ));
}

#[test]
fn the_stored_name_is_trimmed() {
    let command = rename("  Evening calm  ");
    assert_eq!(
        RenameMeditationCommandValidator.normalized_name(&command),
        "Evening calm"
    );
}
