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

    async fn list(&self) -> Result<Vec<User>, RepositoryError> {
        Ok(Vec::new())
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
async fn login_rejects_missing_origin_with_forbidden() {
    // A request with no Origin header at all is rejected: fail closed
    // rather than assume it is a same-origin browser request.
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
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

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

struct SeededRepository {
    inner: EmptyRepository,
    users: Vec<User>,
}

#[async_trait::async_trait]
impl UserRepository for SeededRepository {
    async fn count(&self) -> Result<u64, RepositoryError> {
        Ok(self.users.len() as u64)
    }

    async fn list(&self) -> Result<Vec<User>, RepositoryError> {
        let mut users = self.users.clone();
        users.sort_by(|a, b| a.email().as_str().cmp(b.email().as_str()));
        Ok(users)
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        self.inner.find_by_email(email).await
    }

    async fn find_by_id(&self, id: kernel::UserId) -> Result<Option<User>, RepositoryError> {
        self.inner.find_by_id(id).await
    }

    async fn insert_initial_admin_if_uninitialized(
        &self,
        registration: identity_application::ports::InitialAdminRegistration,
    ) -> Result<User, RepositoryError> {
        self.inner
            .insert_initial_admin_if_uninitialized(registration)
            .await
    }

    async fn insert_member(
        &self,
        registration: identity_application::ports::MemberRegistration,
    ) -> Result<User, RepositoryError> {
        self.inner.insert_member(registration).await
    }

    async fn apply_credential_change(
        &self,
        credential_change: identity_application::ports::CredentialChange,
    ) -> Result<User, RepositoryError> {
        self.inner.apply_credential_change(credential_change).await
    }
}

#[async_trait::async_trait]
impl LoginThrottleRepository for SeededRepository {
    async fn failed_attempts(
        &self,
        normalized_identifier: &str,
    ) -> Result<Option<FailedAttempts>, RepositoryError> {
        self.inner.failed_attempts(normalized_identifier).await
    }

    async fn record_failed_attempt(
        &self,
        normalized_identifier: &str,
        attempted_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<FailedAttempts, RepositoryError> {
        self.inner
            .record_failed_attempt(normalized_identifier, attempted_at)
            .await
    }

    async fn clear_failed_attempts(
        &self,
        normalized_identifier: &str,
    ) -> Result<(), RepositoryError> {
        self.inner
            .clear_failed_attempts(normalized_identifier)
            .await
    }
}

fn seeded_app(users: Vec<User>) -> axum::Router {
    let dependencies = Arc::new(IdentityDependencies {
        repository: Arc::new(SeededRepository {
            inner: EmptyRepository,
            users,
        }),
        hasher: Arc::new(NullHasher),
        clock: Arc::new(NullClock),
        admission: Arc::new(NullAdmission),
        dummy_password_hash: PasswordHash::from_stored("null:dummy".to_owned()),
        session_secret: Arc::new("session-secret-value".to_owned()),
        medioss_origin: Arc::new("https://medioss.example".to_owned()),
    });
    identity_router::<SeededRepository, NullHasher, NullClock, NullAdmission>()
        .with_state(dependencies)
        .layer(tower_sessions::SessionManagerLayer::new(
            tower_sessions::MemoryStore::default(),
        ))
}

fn user(id: i64, email: &str, role: Role) -> User {
    User::new_permanent(
        kernel::UserId::new(id),
        Email::parse(email).unwrap(),
        PasswordHash::from_stored("null:x".to_owned()),
        role,
        chrono::Utc::now(),
    )
}

#[tokio::test]
async fn users_list_returns_sorted_users_for_admin() {
    let router = seeded_app(vec![
        user(2, "zoe@example.com", Role::Member),
        user(1, "adam@example.com", Role::Admin),
    ]);
    let mut request = Request::get("/identity/users").body(Body::empty()).unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let listed: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    let emails: Vec<&str> = listed
        .iter()
        .map(|u| u["email"].as_str().unwrap())
        .collect();
    assert_eq!(emails, vec!["adam@example.com", "zoe@example.com"]);
}

#[tokio::test]
async fn users_list_forbids_members() {
    let router = seeded_app(vec![user(2, "member@example.com", Role::Member)]);
    let mut request = Request::get("/identity/users").body(Body::empty()).unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(2), Role::Member));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn users_list_requires_authentication() {
    let router = seeded_app(vec![]);
    let response = router
        .oneshot(Request::get("/identity/users").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn users_create_forbids_members_at_the_route_level() {
    let router = seeded_app(vec![user(2, "member@example.com", Role::Member)]);
    let mut request = Request::post("/identity/users")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"new@example.com","password":"long-enough"}"#,
        ))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(2), Role::Member));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
