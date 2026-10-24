use super::{CreateProgramCommand, CreateProgramCommandValidator};
use crate::commands::create_program::MAX_COVER_BYTES;
use crate::ApplicationError;

fn command(title: &str, cover_bytes: Option<Vec<u8>>) -> CreateProgramCommand {
    CreateProgramCommand {
        title: title.into(),
        description: None,
        cover_bytes,
    }
}

#[test]
fn accepts_title_and_cover_at_the_limit() {
    let cover = Some(vec![0; MAX_COVER_BYTES]);
    assert_eq!(
        CreateProgramCommandValidator.validate(&command("Calm", cover)),
        Ok(())
    );
}

#[test]
fn rejects_a_blank_title() {
    assert!(matches!(
        CreateProgramCommandValidator.validate(&command(" ", None)),
        Err(ApplicationError::InvalidInput(_))
    ));
}

#[test]
fn rejects_an_oversized_cover() {
    let cover = Some(vec![0; MAX_COVER_BYTES + 1]);
    assert_eq!(
        CreateProgramCommandValidator.validate(&command("Calm", cover)),
        Err(ApplicationError::PayloadTooLarge)
    );
}
