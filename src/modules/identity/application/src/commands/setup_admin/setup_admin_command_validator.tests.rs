use identity_domain::{Email, RawPassword};

use super::{SetupAdminCommand, SetupAdminCommandValidator};

#[test]
fn command_with_parsed_email_and_password_is_accepted() {
    let command = SetupAdminCommand {
        email: Email::parse("admin@local.invalid").unwrap(),
        password: RawPassword::parse("correct-horse-battery").unwrap(),
    };

    assert!(SetupAdminCommandValidator.validate(&command).is_ok());
}
