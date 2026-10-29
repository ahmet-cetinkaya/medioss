use super::{UploadMeditationCommand, UploadMeditationCommandValidator};
use crate::ports::StagedMedia;
use crate::ApplicationError;

const CAP: usize = 1024;

fn validator() -> UploadMeditationCommandValidator {
    UploadMeditationCommandValidator::new(CAP)
}

fn media_of(size_bytes: usize) -> StagedMedia {
    StagedMedia {
        address: "staged".to_owned(),
        size_bytes: size_bytes as u64,
    }
}

fn valid_command() -> UploadMeditationCommand {
    UploadMeditationCommand {
        name: "Morning".to_owned(),
        program_id: 1,
        media: media_of(4),
        description: None,
    }
}

#[test]
fn accepts_a_valid_command() {
    assert!(validator().validate(&valid_command()).is_ok());
}

#[test]
fn rejects_a_blank_name() {
    let command = UploadMeditationCommand {
        name: "   ".to_owned(),
        ..valid_command()
    };

    assert!(matches!(
        validator().validate(&command),
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
        validator().validate(&command),
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
        validator().validate(&command),
        Err(ApplicationError::InvalidDescription)
    );
}

#[test]
fn accepts_a_payload_exactly_at_the_injected_cap() {
    let command = UploadMeditationCommand {
        media: media_of(CAP),
        ..valid_command()
    };

    assert!(validator().validate(&command).is_ok());
}

#[test]
fn rejects_a_payload_over_the_size_cap() {
    let command = UploadMeditationCommand {
        media: media_of(CAP + 1),
        ..valid_command()
    };

    assert_eq!(
        validator().validate(&command),
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

    assert_eq!(validator().normalized_name(&command), "Morning");
    assert_eq!(
        validator().normalized_description(&command),
        Ok(Some("calm".to_owned()))
    );
}
