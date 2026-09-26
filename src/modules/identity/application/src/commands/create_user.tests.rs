use chrono::TimeDelta;
use identity_domain::Role;
use kernel::{Actor, UserId};

use super::create_user;
use crate::{
    test_support::{block_on, email, password, FakeHasher, FakeRepository, FixedClock},
    ApplicationError,
};

#[test]
fn member_actor_is_forbidden_before_password_hashing() {
    let repository = FakeRepository::new();
    let hasher = FakeHasher::new();
    let clock = FixedClock::new();
    let member = Actor::new(UserId::new(7), Role::Member);

    let result = block_on(create_user(
        &repository,
        &hasher,
        &clock,
        &member,
        email(),
        &password(),
    ));

    assert_eq!(result, Err(ApplicationError::Forbidden));
    assert_eq!(hasher.hash_calls(), 0);
    assert_eq!(repository.user_count(), 0);
}

#[test]
fn admin_creates_a_member_with_temporary_credentials() {
    let repository = FakeRepository::new();
    let hasher = FakeHasher::new();
    let clock = FixedClock::new();
    let admin = Actor::new(UserId::new(1), Role::Admin);

    let member = block_on(create_user(
        &repository,
        &hasher,
        &clock,
        &admin,
        email(),
        &password(),
    ))
    .unwrap();

    assert_eq!(member.role(), Role::Member);
    assert!(member.must_change_password());
    assert_eq!(
        member.password_expires_at(),
        Some(clock.now_value() + TimeDelta::hours(24))
    );
}
