use std::sync::Arc;
use std::time::Duration;

use chrono::TimeDelta;
use identity_domain::{required_delay, Email, IdentityError, PasswordHash, Role, User};
use kernel::UserId;

use super::{AuthenticateCommand, AuthenticateCommandHandler, CommandHandler};
use crate::{
    test_support::{
        block_on, email, password, FakeHasher, FakeLoginAdmission, FakeRepository, FixedClock,
    },
    ApplicationError,
};

#[test]
fn authentication_normalizes_email_and_verifies_once() {
    let repository = Arc::new(FakeRepository::with_permanent_admin());
    let hasher = Arc::new(FakeHasher::new());
    let clock = Arc::new(FixedClock::new());
    let admission = Arc::new(FakeLoginAdmission::available());
    let command = service(
        repository.clone(),
        hasher.clone(),
        clock.clone(),
        admission.clone(),
        dummy_hash(),
    );

    let mixed_case_email = format!("  {}  ", email().as_str().to_uppercase());
    let user = block_on(command.handle(login(&mixed_case_email, password().as_str()))).unwrap();

    assert_eq!(user.email(), &email());
    assert_eq!(hasher.verify_calls(), 1);
}

#[test]
fn unknown_and_wrong_passwords_share_the_generic_error_and_one_verify() {
    let repository = Arc::new(FakeRepository::with_permanent_admin());
    let hasher = Arc::new(FakeHasher::new());
    let clock = Arc::new(FixedClock::new());
    let admission = Arc::new(FakeLoginAdmission::available());
    let command = service(
        repository.clone(),
        hasher.clone(),
        clock.clone(),
        admission.clone(),
        dummy_hash(),
    );
    let unknown_identifier = unknown_email("missing");
    let unknown = block_on(command.handle(login(&unknown_identifier, "wrong-password")));
    let wrong = block_on(command.handle(login(email().as_str(), "wrong-password")));

    assert_eq!(unknown, Err(ApplicationError::InvalidCredentials));
    assert_eq!(wrong, Err(ApplicationError::InvalidCredentials));
    assert_eq!(hasher.verify_calls(), 2);
    assert_eq!(
        repository.failed_attempts(&unknown_identifier),
        repository.failed_attempts(email().as_str())
    );
}

#[test]
fn expired_temporary_password_has_the_same_public_failure() {
    let repository = Arc::new(FakeRepository::new());
    let hasher = Arc::new(FakeHasher::new());
    let clock = Arc::new(FixedClock::new());
    let admission = Arc::new(FakeLoginAdmission::available());
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
    let command = service(
        repository.clone(),
        hasher.clone(),
        clock.clone(),
        admission.clone(),
        dummy_hash(),
    );

    let result = block_on(command.handle(login("expired@local.invalid", password().as_str())));

    assert_eq!(result, Err(ApplicationError::InvalidCredentials));
    assert_eq!(hasher.verify_calls(), 1);
    assert_eq!(repository.failed_attempts("expired@local.invalid"), 1);
}

#[test]
fn malformed_email_still_uses_the_fixed_dummy_hash_once() {
    let repository = Arc::new(FakeRepository::new());
    let hasher = Arc::new(FakeHasher::new());
    let clock = Arc::new(FixedClock::new());
    let admission = Arc::new(FakeLoginAdmission::available());
    let dummy_hash_value = dummy_hash();
    let command = service(
        repository.clone(),
        hasher.clone(),
        clock.clone(),
        admission.clone(),
        dummy_hash_value.clone(),
    );

    let result = block_on(command.handle(login(" NOT-AN-EMAIL ", "short")));

    assert_eq!(result, Err(ApplicationError::InvalidCredentials));
    assert_eq!(hasher.verify_calls(), 1);
    assert_eq!(hasher.last_verified_hash(), Some(dummy_hash_value));
    assert_eq!(repository.failed_attempts("not-an-email"), 1);
}

#[test]
fn throttle_state_uses_the_normalized_identifier_for_unknown_users() {
    let repository = Arc::new(FakeRepository::new());
    let hasher = Arc::new(FakeHasher::new());
    let clock = Arc::new(FixedClock::new());
    let admission = Arc::new(FakeLoginAdmission::available());
    let command = service(
        repository.clone(),
        hasher.clone(),
        clock.clone(),
        admission.clone(),
        dummy_hash(),
    );

    let result = block_on(command.handle(login("  MiSsInG-Identifier  ", "wrong-password")));

    assert_eq!(result, Err(ApplicationError::InvalidCredentials));
    assert_eq!(repository.failed_attempts("missing-identifier"), 1);
    assert_eq!(repository.failed_attempts("MiSsInG-Identifier"), 0);
}

#[test]
fn throttle_progression_has_exact_known_and_unknown_public_parity() {
    let repository = Arc::new(FakeRepository::with_permanent_admin());
    let hasher = Arc::new(FakeHasher::new());
    let clock = Arc::new(FixedClock::new());
    let admission = Arc::new(FakeLoginAdmission::available());
    let command = service(
        repository.clone(),
        hasher.clone(),
        clock.clone(),
        admission.clone(),
        dummy_hash(),
    );
    let unknown_identifier = unknown_email("missing");

    for failure_number in 1..=8 {
        let known_failure = block_on(command.handle(login(email().as_str(), "wrong-password")));
        let unknown_failure =
            block_on(command.handle(login(&unknown_identifier, "wrong-password")));
        assert_eq!(
            public_response(&known_failure),
            public_response(&unknown_failure)
        );
        assert_eq!(
            public_response(&known_failure),
            (401, "invalid credentials", None)
        );

        let known_block = block_on(command.handle(login(email().as_str(), "wrong-password")));
        let unknown_block = block_on(command.handle(login(&unknown_identifier, "wrong-password")));
        assert_eq!(
            public_response(&known_block),
            public_response(&unknown_block)
        );
        assert_eq!(
            public_response(&known_block),
            (
                429,
                "login temporarily throttled",
                Some(required_delay_for(failure_number).as_secs()),
            )
        );
        assert_eq!(hasher.verify_calls(), failure_number as usize * 2);

        clock.advance(required_delay_for(failure_number));
    }
}

#[test]
fn global_admission_throttle_is_generic_for_rotating_unknown_identifiers() {
    let repository = Arc::new(FakeRepository::new());
    let hasher = Arc::new(FakeHasher::new());
    let clock = Arc::new(FixedClock::new());
    let admission = Arc::new(FakeLoginAdmission::saturated(Duration::from_secs(3)));
    let command = service(
        repository.clone(),
        hasher.clone(),
        clock.clone(),
        admission.clone(),
        dummy_hash(),
    );

    for identifier in [
        unknown_email("one"),
        unknown_email("two"),
        "not-an-email".to_owned(),
    ] {
        let result = block_on(command.handle(login(&identifier, "wrong-password")));

        assert_eq!(result, Err(throttled_error(Duration::from_secs(3))));
        assert_eq!(repository.failed_attempts(&identifier), 0);
    }
    assert_eq!(hasher.verify_calls(), 0);
}

#[test]
fn successful_authentication_clears_throttle_state() {
    let repository = Arc::new(FakeRepository::with_permanent_admin());
    let hasher = Arc::new(FakeHasher::new());
    let clock = Arc::new(FixedClock::new());
    let admission = Arc::new(FakeLoginAdmission::available());
    let command = service(
        repository.clone(),
        hasher.clone(),
        clock.clone(),
        admission.clone(),
        dummy_hash(),
    );

    assert_eq!(
        block_on(command.handle(login(email().as_str(), "wrong-password"))),
        Err(ApplicationError::InvalidCredentials)
    );
    clock.advance(Duration::from_secs(1));

    let result = block_on(command.handle(login(email().as_str(), password().as_str())));

    assert!(result.is_ok());
    assert_eq!(repository.failed_attempts(email().as_str()), 0);
}

#[test]
fn throttle_allows_success_after_one_hundred_failures_and_the_final_delay() {
    let repository = Arc::new(FakeRepository::with_permanent_admin());
    let hasher = Arc::new(FakeHasher::new());
    let clock = Arc::new(FixedClock::new());
    let admission = Arc::new(FakeLoginAdmission::available());
    let command = service(
        repository.clone(),
        hasher.clone(),
        clock.clone(),
        admission.clone(),
        dummy_hash(),
    );

    for failure_number in 1..=100 {
        let result = block_on(command.handle(login(email().as_str(), "wrong-password")));
        assert_eq!(result, Err(ApplicationError::InvalidCredentials));
        assert_eq!(repository.failed_attempts(email().as_str()), failure_number);

        if failure_number < 100 {
            clock.advance(required_delay_for(failure_number));
        }
    }

    let still_blocked = block_on(command.handle(login(email().as_str(), password().as_str())));
    assert_eq!(still_blocked, Err(throttled_error(Duration::from_secs(64))));
    assert_eq!(hasher.verify_calls(), 100);

    clock.advance(Duration::from_secs(64));
    let recovered = block_on(command.handle(login(email().as_str(), password().as_str())));

    assert!(recovered.is_ok());
    assert_eq!(hasher.verify_calls(), 101);
    assert_eq!(repository.failed_attempts(email().as_str()), 0);
}

fn required_delay_for(failure_number: u64) -> Duration {
    required_delay(failure_number)
}

fn throttled_error(retry_after: Duration) -> ApplicationError {
    ApplicationError::InvalidInput(IdentityError::LoginThrottled { retry_after })
}

fn public_response(result: &Result<User, ApplicationError>) -> (u16, &'static str, Option<u64>) {
    match result {
        Err(ApplicationError::InvalidCredentials) => (401, "invalid credentials", None),
        Err(ApplicationError::InvalidInput(IdentityError::LoginThrottled { retry_after })) => (
            429,
            "login temporarily throttled",
            Some(retry_after.as_secs()),
        ),
        _ => panic!("unexpected authentication result"),
    }
}

fn unknown_email(local_part: &str) -> String {
    [local_part, "example.com"].join("@")
}

fn dummy_hash() -> PasswordHash {
    PasswordHash::from_stored(
        "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQxMjM0NTY3OA$\
         F6Ap1OZqQJ6YxV5lQz0vY7Z5dSZn7zvD2oMZK8Jj9E0"
            .to_owned(),
    )
}

fn service(
    repository: Arc<FakeRepository>,
    hasher: Arc<FakeHasher>,
    clock: Arc<FixedClock>,
    admission: Arc<FakeLoginAdmission>,
    dummy_password_hash: PasswordHash,
) -> AuthenticateCommandHandler {
    AuthenticateCommandHandler::new(
        repository.clone(),
        repository,
        hasher,
        clock,
        admission,
        dummy_password_hash,
    )
}

fn login(identifier: &str, password: &str) -> AuthenticateCommand {
    AuthenticateCommand {
        identifier: identifier.to_owned(),
        password: password.to_owned(),
        bypass_throttle: false,
    }
}
