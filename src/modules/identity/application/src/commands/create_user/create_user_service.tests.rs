use std::sync::Arc;

use chrono::TimeDelta;
use identity_domain::Role;
use kernel::Actor;

use super::{CreateUser, CreateUserHandler, CreateUserService};
use crate::{
    test_support::{block_on, email, password, FakeHasher, FakeRepository, FixedClock},
    ApplicationError,
};

#[test]
fn member_actor_is_forbidden_before_password_hashing() {
    let repository = Arc::new(FakeRepository::new());
    let hasher = Arc::new(FakeHasher::new());
    let clock = Arc::new(FixedClock::new());
    let member = Actor::new(kernel::UserId::new(7), Role::Member);

    let result = block_on(
        CreateUserService::new(repository.clone(), hasher.clone(), clock).handle(CreateUser {
            actor: member,
            email: email(),
            temporary_password: password(),
        }),
    );

    assert_eq!(result, Err(ApplicationError::Forbidden));
    assert_eq!(hasher.hash_calls(), 0);
    assert_eq!(repository.user_count(), 0);
}

#[test]
fn admin_creates_a_member_with_temporary_credentials() {
    let repository = Arc::new(FakeRepository::new());
    let hasher = Arc::new(FakeHasher::new());
    let clock = Arc::new(FixedClock::new());
    let admin = Actor::new(kernel::UserId::new(1), Role::Admin);

    let member = block_on(
        CreateUserService::new(repository.clone(), hasher, clock.clone()).handle(CreateUser {
            actor: admin,
            email: email(),
            temporary_password: password(),
        }),
    )
    .unwrap();

    assert_eq!(member.role(), Role::Member);
    assert!(member.must_change_password());
    assert_eq!(
        member.password_expires_at(),
        Some(clock.now_value() + TimeDelta::hours(24))
    );
}
