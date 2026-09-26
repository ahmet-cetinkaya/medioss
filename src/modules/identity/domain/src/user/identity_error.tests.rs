use std::time::Duration;

use super::*;

#[test]
fn required_identity_failures_are_distinct() {
    assert_ne!(
        IdentityError::InvalidEmail,
        IdentityError::EmailAlreadyExists
    );
    assert_ne!(
        IdentityError::UserNotFound,
        IdentityError::InvalidCredentials
    );
    assert_ne!(IdentityError::WeakPassword, IdentityError::Forbidden);
}

#[test]
fn throttling_carries_a_typed_retry_duration() {
    let retry_after = Duration::from_secs(8);
    let error = IdentityError::LoginThrottled { retry_after };

    assert_eq!(error.retry_after(), Some(retry_after));
}

#[test]
fn setup_completion_has_a_domain_error() {
    assert_eq!(
        IdentityError::SetupAlreadyCompleted.to_string(),
        "initial setup has already been completed"
    );
}
