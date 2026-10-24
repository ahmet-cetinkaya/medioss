use identity_domain::RawPassword;
use kernel::UserId;

use super::{ChangePasswordCommand, ChangePasswordCommandValidator};

#[test]
fn command_with_a_parsed_password_is_accepted() {
    let command = ChangePasswordCommand {
        user_id: UserId::new(1),
        new_password: RawPassword::parse("replacement-password").unwrap(),
    };

    assert!(ChangePasswordCommandValidator.validate(&command).is_ok());
}
