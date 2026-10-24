use super::{ClearLoginThrottleCommand, ClearLoginThrottleCommandValidator};

#[test]
fn any_identifier_is_accepted() {
    let command = ClearLoginThrottleCommand {
        identifier: "Member@Local.Invalid".to_owned(),
    };

    assert!(ClearLoginThrottleCommandValidator
        .validate(&command)
        .is_ok());
}

#[test]
fn normalized_identifier_is_trimmed_and_lowercased() {
    let command = ClearLoginThrottleCommand {
        identifier: "  Member@Local.Invalid ".to_owned(),
    };

    assert_eq!(
        ClearLoginThrottleCommandValidator.normalized_identifier(&command),
        "member@local.invalid"
    );
}
