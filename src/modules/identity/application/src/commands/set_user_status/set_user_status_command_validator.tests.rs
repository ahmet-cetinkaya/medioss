use kernel::{Actor, UserId};

use super::*;

fn command(role: Role, target: i64, disabled: bool) -> SetUserStatusCommand {
    SetUserStatusCommand {
        actor: Actor::new(UserId::new(1), role),
        target: UserId::new(target),
        disabled,
    }
}

#[test]
fn admin_disabling_another_user_is_accepted() {
    let result = SetUserStatusCommandValidator.validate(&command(Role::Admin, 2, true));

    assert_eq!(result, Ok(()));
}

#[test]
fn admin_enabling_oneself_is_accepted() {
    let result = SetUserStatusCommandValidator.validate(&command(Role::Admin, 1, false));

    assert_eq!(result, Ok(()));
}

#[test]
fn member_actor_is_forbidden() {
    let result = SetUserStatusCommandValidator.validate(&command(Role::Member, 2, true));

    assert_eq!(result, Err(ApplicationError::Forbidden));
}

#[test]
fn self_disable_is_refused() {
    let result = SetUserStatusCommandValidator.validate(&command(Role::Admin, 1, true));

    assert_eq!(result, Err(ApplicationError::LastActiveAdmin));
}
