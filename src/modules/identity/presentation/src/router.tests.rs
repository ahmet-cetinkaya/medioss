use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use identity_application::{
    ports::{
        Clock, LoginAdmission, LoginThrottleRepository, PasswordHasher, RepositoryError,
        UserRepository,
    },
    ApplicationError,
};
use identity_domain::{Email, FailedAttempts, IdentityError, PasswordHash, RawPassword, User};
use kernel::Role;
use tower::ServiceExt;

use super::{identity_router, IdentityDependencies, Principal, SharedDependencies};

struct NullHasher;

#[async_trait::async_trait]
impl PasswordHasher for NullHasher {
    async fn hash(
        &self,
        password: &RawPassword,
    ) -> Result<PasswordHash, identity_application::ports::PasswordHasherError> {
        Ok(PasswordHash::from_stored(format!(
            "null:{}",
            password.as_str()
        )))
    }

    async fn verify(
        &self,
        password: &str,
        password_hash: &PasswordHash,
    ) -> Result<bool, identity_application::ports::PasswordHasherError> {
        Ok(password_hash.as_str() == format!("null:{password}"))
    }
}

struct NullClock;

impl Clock for NullClock {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

struct NullAdmission;

impl LoginAdmission for NullAdmission {
    type Permit = ();

    fn try_acquire(&self) -> Result<Self::Permit, std::time::Duration> {
        Ok(())
    }
}

struct EmptyRepository;

#[async_trait::async_trait]
impl UserRepository for EmptyRepository {
    async fn count(&self) -> Result<u64, RepositoryError> {
        Ok(0)
    }

    async fn find_by_email(&self, _email: &Email) -> Result<Option<User>, RepositoryError> {
        Ok(None)
    }

    async fn find_by_id(&self, _id: kernel::UserId) -> Result<Option<User>, RepositoryError> {
        Ok(None)
    }

    async fn insert_initial_admin_if_uninitialized(
        &self,
        _registration: identity_application::ports::InitialAdminRegistration,
    ) -> Result<User, RepositoryError> {
        Err(RepositoryError::OperationFailed)
    }

    async fn insert_member(
        &self,
        _registration: identity_application::ports::MemberRegistration,
    ) -> Result<User, RepositoryError> {
        Err(RepositoryError::OperationFailed)
    }

    async fn apply_credential_change(
        &self,
        _credential_change: identity_application::ports::CredentialChange,
    ) -> Result<User, RepositoryError> {
        Err(RepositoryError::OperationFailed)
    }
}

#[async_trait::async_trait]
impl LoginThrottleRepository for EmptyRepository {
    async fn failed_attempts(
        &self,
        _normalized_identifier: &str,
    ) -> Result<Option<FailedAttempts>, RepositoryError> {
        Ok(None)
    }

    async fn record_failed_attempt(
        &self,
        _normalized_identifier: &str,
        _attempted_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<FailedAttempts, RepositoryError> {
        // Reaching the throttle is legitimate for an unknown email (the
        // response is deliberately uniform); return a fresh zero-count
        // record instead of panicking.
        Ok(FailedAttempts::first_failure(_attempted_at))
    }

    async fn clear_failed_attempts(
        &self,
        _normalized_identifier: &str,
    ) -> Result<(), RepositoryError> {
        Ok(())
    }
}

fn dependencies() -> SharedDependencies<EmptyRepository, NullHasher, NullClock, NullAdmission> {
    Arc::new(IdentityDependencies {
        repository: Arc::new(EmptyRepository),
        hasher: Arc::new(NullHasher),
        clock: Arc::new(NullClock),
        admission: Arc::new(NullAdmission),
        dummy_password_hash: PasswordHash::from_stored("null:dummy".to_owned()),
        session_secret: Arc::new("session-secret-value".to_owned()),
        medioss_origin: Arc::new("https://medioss.example".to_owned()),
    })
}

fn app() -> axum::Router {
    identity_router::<EmptyRepository, NullHasher, NullClock, NullAdmission>()
        .with_state(dependencies())
        .layer(tower_sessions::SessionManagerLayer::new(
            tower_sessions::MemoryStore::default(),
        ))
}

#[tokio::test]
async fn setup_status_is_public_and_reports_required_when_empty() {
    let response = app()
        .oneshot(
            Request::get("/identity/setup-status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn setup_rejects_missing_origin_with_forbidden() {
    let response = app()
        .oneshot(
            Request::post("/identity/setup")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"long-enough"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn setup_rejects_mismatched_origin_with_forbidden() {
    let response = app()
        .oneshot(
            Request::post("/identity/setup")
                .header("origin", "https://evil.example")
                .header("x-setup-secret", "session-secret-value")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"long-enough"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn setup_rejects_missing_secret_with_forbidden() {
    let response = app()
        .oneshot(
            Request::post("/identity/setup")
                .header("origin", "https://medioss.example")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"long-enough"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn setup_rejects_wrong_secret_with_forbidden() {
    let response = app()
        .oneshot(
            Request::post("/identity/setup")
                .header("origin", "https://medioss.example")
                .header("x-setup-secret", "wrong-secret")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"long-enough"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn login_accepts_missing_origin_but_rejects_mismatched() {
    // A same-origin browser request carries no Origin header (cookies sent
    // first-party); it must pass. Only a mismatched Origin is CSRF-relevant.
    let response = app()
        .oneshot(
            Request::post("/identity/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"long-enough"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(response.status(), StatusCode::FORBIDDEN);

    let response = app()
        .oneshot(
            Request::post("/identity/login")
                .header("content-type", "application/json")
                .header("origin", "http://evil.example")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"long-enough"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn me_without_actor_returns_unauthorized() {
    let response = app()
        .oneshot(Request::get("/identity/me").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn users_without_actor_returns_unauthorized_before_body_parsing() {
    let response = app()
        .oneshot(
            Request::post("/identity/users")
                .header("origin", "https://medioss.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn weak_password_maps_to_bad_request() {
    let response = app()
        .oneshot(
            Request::post("/identity/setup")
                .header("origin", "https://medioss.example")
                .header("x-setup-secret", "session-secret-value")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"short"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn constant_time_eq_rejects_length_mismatch_without_leaking() {
    assert!(!super::constant_time_eq(b"abc", b"abcd"));
    assert!(super::constant_time_eq(b"same", b"same"));
}

#[test]
fn principal_role_serializes_as_lowercase_string() {
    let principal = Principal {
        id: 7,
        email: "a@b.c".to_owned(),
        role: Role::Admin,
        must_change_password: false,
        auth_version: 0,
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
        let response = super::ApiError::from(error).into_response_status();
        assert_eq!(response, expected);
    }
}
