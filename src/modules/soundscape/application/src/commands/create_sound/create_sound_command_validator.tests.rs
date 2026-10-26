use super::{CreateSoundCommand, CreateSoundCommandValidator};
use crate::ApplicationError;

const CAP: usize = 1024;

fn validator() -> CreateSoundCommandValidator {
    CreateSoundCommandValidator::new(CAP)
}

fn command(name: &str, icon_key: Option<&str>, size: usize) -> CreateSoundCommand {
    CreateSoundCommand {
        name: name.to_owned(),
        category_id: 1,
        bytes: vec![0; size],
        icon_key: icon_key.map(str::to_owned),
        icon: None,
    }
}

#[test]
fn accepts_a_valid_name_without_icon_key_at_the_size_limit() {
    let command = command("Rain", None, CAP);

    assert!(validator().validate(&command).is_ok());
}

#[test]
fn rejects_a_blank_name() {
    let result = validator().validate(&command("  ", None, 1));

    assert!(matches!(
        result,
        Err(ApplicationError::InvalidInput("sound name"))
    ));
}

#[test]
fn rejects_a_malformed_icon_key() {
    let result = validator().validate(&command("Rain", Some("not a key!"), 1));

    assert!(matches!(
        result,
        Err(ApplicationError::InvalidInput("icon key"))
    ));
}

#[test]
fn rejects_an_upload_over_the_size_limit() {
    let result = validator().validate(&command("Rain", None, CAP + 1));

    assert!(matches!(result, Err(ApplicationError::PayloadTooLarge)));
}
