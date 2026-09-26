use identity_domain::{Email, RawPassword, Role};

use super::setup_admin;
use crate::{
    test_support::{block_on, email, password, FakeHasher, FakeRepository, FixedClock},
    ApplicationError,
};

#[test]
fn setup_creates_a_permanent_admin_through_the_atomic_operation() {
    let repository = FakeRepository::new();
    let hasher = FakeHasher::new();
    let clock = FixedClock::new();

    let admin = block_on(setup_admin(
        &repository,
        &hasher,
        &clock,
        email(),
        &password(),
    ))
    .unwrap();

    assert_eq!(admin.role(), Role::Admin);
    assert!(!admin.must_change_password());
    assert_eq!(admin.password_expires_at(), None);
    assert_eq!(repository.initial_admin_insert_calls(), 1);
    assert_eq!(repository.count_calls(), 0);
}

#[test]
fn setup_refuses_an_initialized_repository_without_count_then_insert() {
    let repository = FakeRepository::with_permanent_admin();
    let hasher = FakeHasher::new();
    let clock = FixedClock::new();

    let result = block_on(setup_admin(
        &repository,
        &hasher,
        &clock,
        Email::parse("second-admin@local.invalid").unwrap(),
        &RawPassword::parse("another-password").unwrap(),
    ));

    assert_eq!(result, Err(ApplicationError::SetupAlreadyCompleted));
    assert_eq!(repository.initial_admin_insert_calls(), 1);
    assert_eq!(repository.count_calls(), 0);
    assert_eq!(repository.user_count(), 1);
}
