use identity_domain::{Email, PasswordHash, Role, User};
use kernel::{Actor, UserId};

use super::{delete_own_account, ensure_can_delete_own_account};
use crate::{
    ports::Clock,
    test_support::{block_on, FakeRepository, FixedClock},
    ApplicationError,
};

fn user(id: i64, role: Role) -> User {
    User::new_permanent(
        UserId::new(id),
        Email::parse(format!("user{id}@example.com")).unwrap(),
        PasswordHash::from_stored("hash".to_owned()),
        role,
        FixedClock::new().now(),
    )
}

#[test]
fn member_deletes_own_account() {
    let repository = FakeRepository::new();
    repository.seed_user(user(1, Role::Admin));
    repository.seed_user(user(2, Role::Member));
    let member = Actor::new(UserId::new(2), Role::Member);

    block_on(delete_own_account(&repository, &member)).unwrap();

    assert_eq!(repository.user_count(), 1);
    assert!(repository.user_by_id(UserId::new(2)).is_none());
}

#[test]
fn last_admin_cannot_delete_themselves() {
    let repository = FakeRepository::new();
    repository.seed_user(user(1, Role::Admin));
    let admin = Actor::new(UserId::new(1), Role::Admin);

    let result = block_on(delete_own_account(&repository, &admin));

    assert_eq!(result, Err(ApplicationError::LastActiveAdmin));
    assert_eq!(repository.user_count(), 1);
}

#[test]
fn admin_deletes_themselves_when_another_admin_remains() {
    let repository = FakeRepository::new();
    repository.seed_user(user(1, Role::Admin));
    repository.seed_user(user(2, Role::Admin));
    let admin = Actor::new(UserId::new(1), Role::Admin);

    block_on(delete_own_account(&repository, &admin)).unwrap();

    assert_eq!(repository.user_count(), 1);
    assert!(repository.user_by_id(UserId::new(1)).is_none());
}

#[test]
fn ensure_refuses_last_admin_without_deleting() {
    let repository = FakeRepository::new();
    repository.seed_user(user(1, Role::Admin));
    let admin = Actor::new(UserId::new(1), Role::Admin);

    let result = block_on(ensure_can_delete_own_account(&repository, &admin));

    assert_eq!(result, Err(ApplicationError::LastActiveAdmin));
    assert_eq!(repository.user_count(), 1);
}
