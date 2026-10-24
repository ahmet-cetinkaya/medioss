use std::sync::Arc;

use identity_domain::{Email, PasswordHash, Role, User};
use kernel::UserId;

use super::ensure_not_last_active_admin;
use crate::{
    ports::Clock as _,
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
fn a_disabled_admin_may_be_removed() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(1, Role::Admin, true));

    block_on(ensure_not_last_active_admin(
        repository.as_ref(),
        &user(1, Role::Admin, true),
    ))
    .unwrap();
}

#[test]
fn the_only_active_admin_may_not_be_removed() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(1, Role::Admin, false));

    assert_eq!(
        block_on(ensure_not_last_active_admin(
            repository.as_ref(),
            &user(1, Role::Admin, false)
        ))
        .unwrap_err(),
        ApplicationError::LastActiveAdmin
    );
}

#[test]
fn members_are_never_blocked_by_the_guard() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(2, Role::Member, false));

    block_on(ensure_not_last_active_admin(
        repository.as_ref(),
        &user(2, Role::Member, false),
    ))
    .unwrap();
}
