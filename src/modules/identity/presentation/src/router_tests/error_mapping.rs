use axum::http::StatusCode;
use identity_application::ApplicationError;
use identity_domain::IdentityError;
use kernel::Role;

use crate::Principal;

#[test]
fn constant_time_eq_rejects_length_mismatch_without_leaking() {
    assert!(!crate::origin_guard::constant_time_eq(b"abc", b"abcd"));
    assert!(crate::origin_guard::constant_time_eq(b"same", b"same"));
}

#[test]
fn principal_role_serializes_as_lowercase_string() {
    let principal = Principal {
        id: 7,
        email: "a@b.c".to_owned(),
        role: Role::Admin,
        must_change_password: false,
        auth_version: 0,
        display_name: None,
        has_avatar: false,
    };
    assert_eq!(principal.role, Role::Admin);
}

#[test]
fn api_error_mapping_table() {
    let cases: Vec<(ApplicationError, StatusCode)> = vec![
        (
            ApplicationError::InvalidCredentials,
            StatusCode::UNAUTHORIZED,
        ),
        (ApplicationError::Forbidden, StatusCode::FORBIDDEN),
        (
            ApplicationError::SetupAlreadyCompleted,
            StatusCode::CONFLICT,
        ),
        (ApplicationError::EmailAlreadyExists, StatusCode::CONFLICT),
        (ApplicationError::UserNotFound, StatusCode::UNAUTHORIZED),
        (
            ApplicationError::ProcessingUnavailable,
            StatusCode::SERVICE_UNAVAILABLE,
        ),
        (
            ApplicationError::PasswordHasher,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (
            ApplicationError::Repository,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (
            ApplicationError::InvalidInput(IdentityError::WeakPassword),
            StatusCode::BAD_REQUEST,
        ),
        (
            ApplicationError::InvalidInput(IdentityError::LoginThrottled {
                retry_after: std::time::Duration::from_secs(3),
            }),
            StatusCode::TOO_MANY_REQUESTS,
        ),
    ];
    for (error, expected) in cases {
        let response = crate::api_error::ApiError::from(error).into_response_status();
        assert_eq!(response, expected);
    }
}
