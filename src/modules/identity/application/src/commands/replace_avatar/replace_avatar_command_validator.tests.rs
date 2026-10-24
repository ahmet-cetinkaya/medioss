use kernel::{Actor, Role, UserId};

use super::{ReplaceAvatarCommand, ReplaceAvatarCommandValidator};

#[test]
fn accepts_any_bytes_because_the_store_owns_content_rules() {
    let command = ReplaceAvatarCommand {
        actor: Actor::new(UserId::new(2), Role::Member),
        bytes: Vec::new(),
    };

    assert!(ReplaceAvatarCommandValidator.validate(&command).is_ok());
}
