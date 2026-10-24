use identity_domain::DisplayName;
use kernel::UserId;

use super::{SetDisplayNameCommand, SetDisplayNameCommandValidator};

#[test]
fn accepts_a_name() {
    let command = SetDisplayNameCommand {
        user_id: UserId::new(2),
        display_name: Some(DisplayName::parse("Ada").unwrap()),
    };

    assert!(SetDisplayNameCommandValidator.validate(&command).is_ok());
}

#[test]
fn accepts_clearing_the_name() {
    let command = SetDisplayNameCommand {
        user_id: UserId::new(2),
        display_name: None,
    };

    assert!(SetDisplayNameCommandValidator.validate(&command).is_ok());
}
