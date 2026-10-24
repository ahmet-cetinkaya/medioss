use super::{AuthenticateCommand, AuthenticateCommandValidator};

#[test]
fn raw_identifier_and_password_are_accepted_because_they_are_normalized_later() {
    let command = AuthenticateCommand {
        identifier: "  Member@Local.Invalid ".to_owned(),
        password: "any-password".to_owned(),
        bypass_throttle: false,
    };

    assert!(AuthenticateCommandValidator.validate(&command).is_ok());
}

#[test]
fn normalized_identifier_is_trimmed_and_lowercased() {
    let command = AuthenticateCommand {
        identifier: "  Member@Local.Invalid ".to_owned(),
        password: "any-password".to_owned(),
        bypass_throttle: false,
    };

    assert_eq!(
        AuthenticateCommandValidator.normalized_identifier(&command),
        "member@local.invalid"
    );
}
