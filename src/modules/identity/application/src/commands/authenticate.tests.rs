use chrono::TimeDelta;
use identity_domain::{Email, PasswordHash, Role, User};
use kernel::UserId;

use super::Authenticate;
use crate::{
    test_support::{block_on, email, password, FakeHasher, FakeRepository, FixedClock},
    ApplicationError,
};

#[test]
fn authentication_normalizes_email_and_verifies_once() {
    let repository = FakeRepository::with_permanent_admin();
    let hasher = FakeHasher::new();
    let clock = FixedClock::new();
    let command = Authenticate::new(&repository, &hasher, &clock, dummy_hash());

    let user = block_on(command.execute("  ADMIN@LOCAL.INVALID ", password().as_str())).unwrap();

    assert_eq!(user.email(), &email());
    assert_eq!(hasher.verify_calls(), 1);
}

#[test]
fn unknown_and_wrong_passwords_share_the_generic_error_and_one_verify() {
    let repository = FakeRepository::with_permanent_admin();
    let hasher = FakeHasher::new();
    let clock = FixedClock::new();
    let command = Authenticate::new(&repository, &hasher, &clock, dummy_hash());
    let unknown = block_on(command.execute("missing@local.invalid", "wrong-password"));
    let wrong = block_on(command.execute(email().as_str(), "wrong-password"));

    assert_eq!(unknown, Err(ApplicationError::InvalidCredentials));
    assert_eq!(wrong, Err(ApplicationError::InvalidCredentials));
    assert_eq!(hasher.verify_calls(), 2);
    assert_eq!(
        repository.failed_attempts("missing@local.invalid"),
        repository.failed_attempts(email().as_str())
    );
}

#[test]
fn expired_temporary_password_has_the_same_public_failure() {
    let repository = FakeRepository::new();
    let hasher = FakeHasher::new();
    let clock = FixedClock::new();
    let created_at = clock.now_value() - TimeDelta::hours(25);
    let expired = User::new_temporary(
        UserId::new(9),
        Email::parse("expired@local.invalid").unwrap(),
        FakeHasher::hash_for(&password()),
        Role::Member,
        created_at,
    )
    .unwrap();
    repository.seed_user(expired);
    let command = Authenticate::new(&repository, &hasher, &clock, dummy_hash());

    let result = block_on(command.execute("expired@local.invalid", password().as_str()));

    assert_eq!(result, Err(ApplicationError::InvalidCredentials));
    assert_eq!(hasher.verify_calls(), 1);
    assert_eq!(repository.failed_attempts("expired@local.invalid"), 1);
}

#[test]
fn malformed_email_still_uses_the_fixed_dummy_hash_once() {
    let repository = FakeRepository::new();
    let hasher = FakeHasher::new();
    let clock = FixedClock::new();
    let dummy_hash = dummy_hash();
    let command = Authenticate::new(&repository, &hasher, &clock, dummy_hash.clone());

    let result = block_on(command.execute(" NOT-AN-EMAIL ", "short"));

    assert_eq!(result, Err(ApplicationError::InvalidCredentials));
    assert_eq!(hasher.verify_calls(), 1);
    assert_eq!(hasher.last_verified_hash(), Some(dummy_hash));
    assert_eq!(repository.failed_attempts("not-an-email"), 1);
}

fn dummy_hash() -> PasswordHash {
    PasswordHash::from_stored(
        "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQxMjM0NTY3OA$\
         F6Ap1OZqQJ6YxV5lQz0vY7Z5dSZn7zvD2oMZK8Jj9E0"
            .to_owned(),
    )
}
