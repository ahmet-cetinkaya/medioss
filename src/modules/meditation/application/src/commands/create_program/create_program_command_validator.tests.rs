use super::{CreateProgramCommand, CreateProgramCommandValidator};
use crate::ApplicationError;

const CAP: usize = 1024;

fn validator() -> CreateProgramCommandValidator {
    CreateProgramCommandValidator::new(CAP)
}

fn command(title: &str, cover_bytes: Option<Vec<u8>>) -> CreateProgramCommand {
    CreateProgramCommand {
        title: title.into(),
        description: None,
        cover_bytes,
    }
}

#[test]
fn accepts_title_and_cover_at_the_limit() {
    let cover = Some(vec![0; CAP]);
    assert_eq!(validator().validate(&command("Calm", cover)), Ok(()));
}

#[test]
fn rejects_a_blank_title() {
    assert!(matches!(
        validator().validate(&command(" ", None)),
        Err(ApplicationError::InvalidInput(_))
    ));
}

#[test]
fn rejects_an_oversized_cover() {
    let cover = Some(vec![0; CAP + 1]);
    assert_eq!(
        validator().validate(&command("Calm", cover)),
        Err(ApplicationError::PayloadTooLarge)
    );
}
