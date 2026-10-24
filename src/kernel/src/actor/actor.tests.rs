use super::Actor;
use crate::actor::role::Role;
use crate::actor::user_id::UserId;

#[test]
fn exposes_the_user_and_role_it_was_built_with() {
    let actor = Actor::new(UserId::new(7), Role::Admin);

    assert_eq!(actor.user_id(), UserId::new(7));
    assert_eq!(actor.role(), Role::Admin);
}

#[test]
fn actors_compare_by_user_and_role() {
    let member = Actor::new(UserId::new(1), Role::Member);

    assert_eq!(member, Actor::new(UserId::new(1), Role::Member));
    assert_ne!(member, Actor::new(UserId::new(1), Role::Admin));
    assert_ne!(member, Actor::new(UserId::new(2), Role::Member));
}
