use std::sync::Arc;

use chrono::TimeDelta;
use identity_domain::{Email, RawPassword, Role, User};
use kernel::UserId;

use super::{ChangePassword, ChangePasswordHandler, ChangePasswordService};
use crate::{
    test_support::{block_on, FakeHasher, FakeRepository, FixedClock},
    ApplicationError,
};

#[test]
fn credential_change_is_applied_as_one_atomic_mutation() {
    let repository = Arc::new(FakeRepository::new());
    let hasher = Arc::new(FakeHasher::new());
    let user = temporary_user();
    let actor = user.actor();
    repository.seed_user(user);
    repository.set_failed_attempts("member@local.invalid", 4);
    let new_password = RawPassword::parse("replacement-password").unwrap();

    let changed = block_on(change_password(
        repository.clone(),
        hasher,
        actor.user_id(),
        new_password,
    ))
    .unwrap();

    assert_eq!(
        changed.password_hash(),
        &FakeHasher::hash_for(&RawPassword::parse("replacement-password").unwrap())
    );
    assert_eq!(changed.auth_version(), 1);
    assert!(!changed.must_change_password());
    assert_eq!(changed.password_expires_at(), None);
    assert_eq!(repository.failed_attempts("member@local.invalid"), 0);
    assert_eq!(repository.credential_change_calls(), 1);
}

#[test]
fn repository_failure_rolls_back_the_entire_credential_change() {
    let repository = Arc::new(FakeRepository::new());
    let hasher = Arc::new(FakeHasher::new());
    let original = temporary_user();
    let actor = original.actor();
    repository.seed_user(original.clone());
    repository.set_failed_attempts("member@local.invalid", 3);
    repository.fail_next_credential_change();

    let result = block_on(change_password(
        repository.clone(),
        hasher,
        actor.user_id(),
        RawPassword::parse("replacement-password").unwrap(),
    ));

    assert_eq!(result, Err(ApplicationError::Repository));
    assert_eq!(repository.user_by_email("member@local.invalid"), original);
    assert_eq!(repository.failed_attempts("member@local.invalid"), 3);
}

async fn change_password(
    repository: Arc<FakeRepository>,
    hasher: Arc<FakeHasher>,
    user_id: UserId,
    new_password: RawPassword,
) -> Result<User, ApplicationError> {
    ChangePasswordService::new(repository, hasher)
        .handle(ChangePassword {
            user_id,
            new_password,
        })
        .await
}

fn temporary_user() -> User {
    User::new_temporary(
        UserId::new(2),
        Email::parse("member@local.invalid").unwrap(),
        FakeHasher::hash_for(&RawPassword::parse("temporary-password").unwrap()),
        Role::Member,
        FixedClock::new().now_value() - TimeDelta::hours(1),
    )
    .unwrap()
}
