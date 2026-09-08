use chrono::TimeDelta;
use identity_domain::{Email, RawPassword, Role, User};
use kernel::UserId;

use super::change_password;
use crate::{
    test_support::{block_on, FakeHasher, FakeRepository, FixedClock},
    ApplicationError,
};

#[test]
fn credential_change_is_applied_as_one_atomic_mutation() {
    let repository = FakeRepository::new();
    let hasher = FakeHasher::new();
    let clock = FixedClock::new();
    let user = temporary_user(&clock);
    let actor = user.actor();
    repository.seed_user(user);
    repository.set_failed_attempts("member@local.invalid", 4);
    let new_password = RawPassword::parse("replacement-password").unwrap();

    let changed = block_on(change_password(&repository, &hasher, &actor, &new_password)).unwrap();

    assert_eq!(
        changed.password_hash(),
        &FakeHasher::hash_for(&new_password)
    );
    assert_eq!(changed.auth_version(), 1);
    assert!(!changed.must_change_password());
    assert_eq!(changed.password_expires_at(), None);
    assert_eq!(repository.failed_attempts("member@local.invalid"), 0);
    assert_eq!(repository.credential_change_calls(), 1);
}

#[test]
fn repository_failure_rolls_back_the_entire_credential_change() {
    let repository = FakeRepository::new();
    let hasher = FakeHasher::new();
    let clock = FixedClock::new();
    let original = temporary_user(&clock);
    let actor = original.actor();
    repository.seed_user(original.clone());
    repository.set_failed_attempts("member@local.invalid", 3);
    repository.fail_next_credential_change();

    let result = block_on(change_password(
        &repository,
        &hasher,
        &actor,
        &RawPassword::parse("replacement-password").unwrap(),
    ));

    assert_eq!(result, Err(ApplicationError::Repository));
    assert_eq!(repository.user_by_email("member@local.invalid"), original);
    assert_eq!(repository.failed_attempts("member@local.invalid"), 3);
}

fn temporary_user(clock: &FixedClock) -> User {
    User::new_temporary(
        UserId::new(2),
        Email::parse("member@local.invalid").unwrap(),
        FakeHasher::hash_for(&RawPassword::parse("temporary-password").unwrap()),
        Role::Member,
        clock.now_value() - TimeDelta::hours(1),
    )
    .unwrap()
}
