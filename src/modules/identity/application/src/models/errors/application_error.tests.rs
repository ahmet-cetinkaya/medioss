use std::time::Duration;

use identity_domain::IdentityError;

use super::ApplicationError;
use crate::ports::{AvatarStoreError, PasswordHasherError, RepositoryError};

#[test]
fn display_messages_are_stable() {
    assert_eq!(
        ApplicationError::InvalidCredentials.to_string(),
        "invalid credentials"
    );
    assert_eq!(
        ApplicationError::Forbidden.to_string(),
        "the actor is not allowed to perform this operation"
    );
    assert_eq!(
        ApplicationError::SetupAlreadyCompleted.to_string(),
        "initial setup has already been completed"
    );
    assert_eq!(
        ApplicationError::EmailAlreadyExists.to_string(),
        "email already exists"
    );
    assert_eq!(
        ApplicationError::UserNotFound.to_string(),
        "user was not found"
    );
    assert_eq!(
        ApplicationError::PasswordHasher.to_string(),
        "password hashing operation failed"
    );
    assert_eq!(
        ApplicationError::Repository.to_string(),
        "repository operation failed"
    );
    assert_eq!(
        ApplicationError::ProcessingUnavailable.to_string(),
        "avatar processing is temporarily unavailable"
    );
    assert_eq!(
        ApplicationError::LastActiveAdmin.to_string(),
        "operation would leave the instance without an active admin"
    );
    assert_eq!(
        ApplicationError::PayloadTooLarge.to_string(),
        "request payload is too large"
    );
    assert_eq!(
        ApplicationError::UnsupportedMedia.to_string(),
        "uploaded image is not a supported media type"
    );
    assert_eq!(
        ApplicationError::InvalidInput(IdentityError::InvalidEmail).to_string(),
        "application input is invalid: email is invalid"
    );
}

#[test]
fn identity_errors_map_to_invalid_input() {
    let throttled = ApplicationError::from(IdentityError::LoginThrottled {
        retry_after: Duration::from_secs(30),
    });
    assert_eq!(
        throttled,
        ApplicationError::InvalidInput(IdentityError::LoginThrottled {
            retry_after: Duration::from_secs(30)
        })
    );
}

#[test]
fn password_hasher_errors_collapse_to_one_variant() {
    assert_eq!(
        ApplicationError::from(PasswordHasherError),
        ApplicationError::PasswordHasher
    );
}

#[test]
fn avatar_store_errors_map_by_kind() {
    assert_eq!(
        ApplicationError::from(AvatarStoreError::InvalidPayload),
        ApplicationError::UnsupportedMedia
    );
    assert_eq!(
        ApplicationError::from(AvatarStoreError::OutputTooLarge),
        ApplicationError::PayloadTooLarge
    );
    assert_eq!(
        ApplicationError::from(AvatarStoreError::WorkerFailed),
        ApplicationError::Repository
    );
    assert_eq!(
        ApplicationError::from(AvatarStoreError::Busy),
        ApplicationError::ProcessingUnavailable
    );
    assert_eq!(
        ApplicationError::from(AvatarStoreError::Storage),
        ApplicationError::Repository
    );
}

#[test]
fn repository_errors_map_by_kind() {
    assert_eq!(
        ApplicationError::from(RepositoryError::SetupAlreadyCompleted),
        ApplicationError::SetupAlreadyCompleted
    );
    assert_eq!(
        ApplicationError::from(RepositoryError::EmailAlreadyExists),
        ApplicationError::EmailAlreadyExists
    );
    assert_eq!(
        ApplicationError::from(RepositoryError::UserNotFound),
        ApplicationError::UserNotFound
    );
    assert_eq!(
        ApplicationError::from(RepositoryError::AuthVersionOverflow),
        ApplicationError::Repository
    );
    assert_eq!(
        ApplicationError::from(RepositoryError::OperationFailed),
        ApplicationError::Repository
    );
}
