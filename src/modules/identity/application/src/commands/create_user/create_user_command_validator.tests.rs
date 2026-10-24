use kernel::{Actor, UserId};

use super::*;
use crate::test_support::{email, password};

fn command_by(role: Role) -> CreateUserCommand {
    CreateUserCommand {
        actor: Actor::new(UserId::new(1), role),
        email: email(),
        temporary_password: password(),
    }
}

#[test]
fn admin_actor_is_accepted() {
    let result = CreateUserCommandValidator.validate(&command_by(Role::Admin));

    assert_eq!(result, Ok(()));
}

#[test]
fn member_actor_is_forbidden() {
    let result = CreateUserCommandValidator.validate(&command_by(Role::Member));

    assert_eq!(result, Err(ApplicationError::Forbidden));
}
