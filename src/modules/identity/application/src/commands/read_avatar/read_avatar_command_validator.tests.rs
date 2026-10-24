use kernel::{Actor, Role, UserId};

use super::{ReadAvatarCommand, ReadAvatarCommandValidator};
use crate::ApplicationError;

fn command(requested: i64, reader_id: i64, reader_role: Role) -> ReadAvatarCommand {
    ReadAvatarCommand {
        requested: UserId::new(requested),
        reader: Actor::new(UserId::new(reader_id), reader_role),
    }
}

#[test]
fn accepts_the_owner() {
    assert!(ReadAvatarCommandValidator
        .validate(&command(2, 2, Role::Member))
        .is_ok());
}

#[test]
fn accepts_an_admin_reading_another_user() {
    assert!(ReadAvatarCommandValidator
        .validate(&command(2, 1, Role::Admin))
        .is_ok());
}

#[test]
fn rejects_a_non_admin_reading_another_user() {
    assert!(matches!(
        ReadAvatarCommandValidator.validate(&command(2, 3, Role::Member)),
        Err(ApplicationError::Forbidden)
    ));
}
