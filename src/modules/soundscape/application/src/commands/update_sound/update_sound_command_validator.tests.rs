use super::{UpdateSoundCommand, UpdateSoundCommandValidator};
use crate::ApplicationError;

fn command(name: &str, icon_key: Option<&str>) -> UpdateSoundCommand {
    UpdateSoundCommand {
        id: 1,
        name: name.to_owned(),
        category_id: 1,
        icon_key: icon_key.map(str::to_owned),
    }
}

#[test]
fn accepts_a_valid_name_and_no_icon_key() {
    assert!(UpdateSoundCommandValidator
        .validate(&command("Rain", None))
        .is_ok());
}

#[test]
fn rejects_a_blank_name() {
    let result = UpdateSoundCommandValidator.validate(&command("   ", None));

    assert!(matches!(
        result,
        Err(ApplicationError::InvalidInput("sound name"))
    ));
}

#[test]
fn rejects_a_malformed_icon_key() {
    let result = UpdateSoundCommandValidator.validate(&command("Rain", Some("not a key!")));

    assert!(matches!(
        result,
        Err(ApplicationError::InvalidInput("icon key"))
    ));
}
