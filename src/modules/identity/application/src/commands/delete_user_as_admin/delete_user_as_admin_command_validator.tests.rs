use kernel::{Actor, UserId};

use super::*;

fn command_by(role: Role) -> DeleteUserAsAdminCommand {
    DeleteUserAsAdminCommand {
        actor: Actor::new(UserId::new(1), role),
        target: UserId::new(2),
    }
}

#[test]
fn admin_actor_is_accepted() {
    let result = DeleteUserAsAdminCommandValidator.validate(&command_by(Role::Admin));

    assert_eq!(result, Ok(()));
}

#[test]
fn member_actor_is_forbidden() {
    let result = DeleteUserAsAdminCommandValidator.validate(&command_by(Role::Member));

    assert_eq!(result, Err(ApplicationError::Forbidden));
}
