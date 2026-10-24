use kernel::{Actor, Role, UserId};

use super::{ClearAvatarCommand, ClearAvatarCommandValidator};

#[test]
fn accepts_any_actor_because_there_is_no_input_rule() {
    let command = ClearAvatarCommand {
        actor: Actor::new(UserId::new(2), Role::Member),
    };

    assert!(ClearAvatarCommandValidator.validate(&command).is_ok());
}
