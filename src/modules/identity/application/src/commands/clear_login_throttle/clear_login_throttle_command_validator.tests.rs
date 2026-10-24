use super::{ClearLoginThrottleCommand, ClearLoginThrottleCommandValidator};

#[test]
fn any_identifier_is_accepted_because_it_is_normalized_later() {
    let command = ClearLoginThrottleCommand {
        identifier: "Member@Local.Invalid".to_owned(),
    };

    assert!(ClearLoginThrottleCommandValidator
        .validate(&command)
        .is_ok());
}
