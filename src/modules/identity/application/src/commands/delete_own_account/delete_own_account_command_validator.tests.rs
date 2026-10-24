use identity_domain::Role;
use kernel::{Actor, UserId};

use super::*;

#[test]
fn any_actor_is_accepted() {
    for role in [Role::Admin, Role::Member] {
        let command = DeleteOwnAccountCommand {
            actor: Actor::new(UserId::new(1), role),
        };

        assert_eq!(DeleteOwnAccountCommandValidator.validate(&command), Ok(()));
    }
}
