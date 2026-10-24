use super::super::upload_meditation_command::MAX_UPLOAD_BYTES;
use super::{UploadMeditationCommand, UploadMeditationCommandValidator};
use crate::ApplicationError;

fn valid_command() -> UploadMeditationCommand {
    UploadMeditationCommand {
        name: "Morning".to_owned(),
        program_id: 1,
        bytes: vec![0; 4],
        description: None,
    }
}

#[test]
fn accepts_a_valid_command() {
    assert!(UploadMeditationCommandValidator
        .validate(&valid_command())
        .is_ok());
}

#[test]
fn rejects_a_blank_name() {
    let command = UploadMeditationCommand {
        name: "   ".to_owned(),
        ..valid_command()
    };

    assert!(matches!(
        UploadMeditationCommandValidator.validate(&command),
        Err(ApplicationError::InvalidInput(_))
    ));
}

#[test]
fn rejects_an_overlong_name() {
    let command = UploadMeditationCommand {
        name: "x".repeat(crate::services::MAX_NAME_LENGTH + 1),
        ..valid_command()
    };

    assert!(matches!(
        UploadMeditationCommandValidator.validate(&command),
        Err(ApplicationError::InvalidInput(_))
    ));
}

#[test]
fn rejects_an_overlong_description() {
    let command = UploadMeditationCommand {
        description: Some("x".repeat(meditation_domain::MAX_DESCRIPTION_LENGTH + 1)),
        ..valid_command()
    };

    assert_eq!(
        UploadMeditationCommandValidator.validate(&command),
        Err(ApplicationError::InvalidDescription)
    );
}

#[test]
fn rejects_a_payload_over_the_size_cap() {
    let command = UploadMeditationCommand {
        bytes: vec![0; MAX_UPLOAD_BYTES + 1],
        ..valid_command()
    };

    assert_eq!(
        UploadMeditationCommandValidator.validate(&command),
        Err(ApplicationError::PayloadTooLarge)
    );
}

#[test]
fn normalizes_name_and_description_by_trimming() {
    let command = UploadMeditationCommand {
        name: "  Morning ".to_owned(),
        description: Some(" calm ".to_owned()),
        ..valid_command()
    };

    assert_eq!(
        UploadMeditationCommandValidator.normalized_name(&command),
        "Morning"
    );
    assert_eq!(
        UploadMeditationCommandValidator.normalized_description(&command),
        Ok(Some("calm".to_owned()))
    );
}
