use identity_domain::{Email, PasswordHash, Role, User};
use kernel::{Actor, UserId};

use super::delete_user_as_admin;
use crate::{
    ports::Clock,
    test_support::{block_on, FakeRepository, FixedClock},
    ApplicationError,
};

fn user(id: i64, role: Role, disabled: bool) -> User {
    User::new_permanent(
        UserId::new(id),
        Email::parse(format!("user{id}@example.com")).unwrap(),
        PasswordHash::from_stored("hash".to_owned()),
        role,
        FixedClock::new().now(),
    )
    .with_disabled(disabled)
}

#[test]
fn member_actor_is_forbidden() {
    let repository = FakeRepository::new();
    repository.seed_user(user(2, Role::Member, false));
    let member = Actor::new(UserId::new(9), Role::Member);

    let result = block_on(delete_user_as_admin(&repository, &member, UserId::new(2)));

    assert_eq!(result, Err(ApplicationError::Forbidden));
    assert_eq!(repository.user_count(), 1);
}

#[test]
fn deleting_the_last_active_admin_is_refused() {
    let repository = FakeRepository::new();
    repository.seed_user(user(1, Role::Admin, false));
    let admin = Actor::new(UserId::new(1), Role::Admin);

    let result = block_on(delete_user_as_admin(&repository, &admin, UserId::new(1)));

    assert_eq!(result, Err(ApplicationError::LastActiveAdmin));
    assert_eq!(repository.user_count(), 1);
}

#[test]
fn admin_deletes_a_member() {
    let repository = FakeRepository::new();
    repository.seed_user(user(1, Role::Admin, false));
    repository.seed_user(user(2, Role::Member, false));
    let admin = Actor::new(UserId::new(1), Role::Admin);

    block_on(delete_user_as_admin(&repository, &admin, UserId::new(2))).unwrap();

    assert_eq!(repository.user_count(), 1);
    assert!(repository.user_by_id(UserId::new(2)).is_none());
}

#[test]
fn unknown_target_is_not_found() {
    let repository = FakeRepository::new();
    repository.seed_user(user(1, Role::Admin, false));
    let admin = Actor::new(UserId::new(1), Role::Admin);

    let result = block_on(delete_user_as_admin(&repository, &admin, UserId::new(99)));

    assert_eq!(result, Err(ApplicationError::UserNotFound));
}
