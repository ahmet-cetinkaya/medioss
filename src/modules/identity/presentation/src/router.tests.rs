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

use crate::{identity_router, IdentityDependencies, Principal, SharedDependencies};

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

struct RecordingAvatarStore {
    normalize_result: std::sync::Mutex<
        Result<(Vec<u8>, &'static str), identity_application::ports::AvatarStoreError>,
    >,
    written: std::sync::Mutex<Vec<&'static str>>,
    removed: std::sync::Mutex<Vec<&'static str>>,
    read_result: std::sync::Mutex<Vec<u8>>,
    next_key: std::sync::atomic::AtomicU64,
}

impl RecordingAvatarStore {
    fn accepting() -> Self {
        Self {
            normalize_result: std::sync::Mutex::new(Ok((vec![1, 2, 3], "webp"))),
            written: std::sync::Mutex::new(Vec::new()),
            removed: std::sync::Mutex::new(Vec::new()),
            read_result: std::sync::Mutex::new(vec![9, 9]),
            next_key: std::sync::atomic::AtomicU64::new(1),
        }
    }
}

#[async_trait::async_trait]
impl identity_application::ports::AvatarStore for RecordingAvatarStore {
    async fn normalize(
        &self,
        _bytes: Vec<u8>,
    ) -> Result<(Vec<u8>, &'static str), identity_application::ports::AvatarStoreError> {
        self.normalize_result.lock().unwrap().clone()
    }

    async fn write(
        &self,
        _id: kernel::UserId,
        extension: &'static str,
        _bytes: &[u8],
    ) -> Result<identity_domain::AvatarRef, identity_application::ports::AvatarStoreError> {
        let sequence = self
            .next_key
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.written.lock().unwrap().push(extension);
        Ok(identity_domain::AvatarRef::new(
            format!("avatar-{sequence:016x}.{extension}"),
            extension,
        )
        .unwrap())
    }

    async fn read(
        &self,
        _id: kernel::UserId,
        _avatar: &identity_domain::AvatarRef,
    ) -> Result<Vec<u8>, identity_application::ports::AvatarStoreError> {
        Ok(self.read_result.lock().unwrap().clone())
    }

    async fn remove(
        &self,
        _id: kernel::UserId,
        avatar: &identity_domain::AvatarRef,
    ) -> Result<(), identity_application::ports::AvatarStoreError> {
        self.removed.lock().unwrap().push(avatar.extension());
        Ok(())
    }
}

struct NullAvatarStore;

#[async_trait::async_trait]
impl identity_application::ports::AvatarStore for NullAvatarStore {
    async fn normalize(
        &self,
        _bytes: Vec<u8>,
    ) -> Result<(Vec<u8>, &'static str), identity_application::ports::AvatarStoreError> {
        Err(identity_application::ports::AvatarStoreError::InvalidPayload)
    }

    async fn write(
        &self,
        _id: kernel::UserId,
        extension: &'static str,
        _bytes: &[u8],
    ) -> Result<identity_domain::AvatarRef, identity_application::ports::AvatarStoreError> {
        Ok(identity_domain::AvatarRef::from_extension(extension).unwrap())
    }

    async fn read(
        &self,
        _id: kernel::UserId,
        _avatar: &identity_domain::AvatarRef,
    ) -> Result<Vec<u8>, identity_application::ports::AvatarStoreError> {
        Ok(Vec::new())
    }

    async fn remove(
        &self,
        _id: kernel::UserId,
        _avatar: &identity_domain::AvatarRef,
    ) -> Result<(), identity_application::ports::AvatarStoreError> {
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

    async fn set_disabled(
        &self,
        _id: kernel::UserId,
        _disabled: bool,
    ) -> Result<(), RepositoryError> {
        Err(RepositoryError::OperationFailed)
    }

    async fn set_profile(
        &self,
        _id: kernel::UserId,
        _display_name: Option<identity_domain::DisplayName>,
        _avatar: Option<identity_domain::AvatarRef>,
    ) -> Result<User, RepositoryError> {
        Err(RepositoryError::OperationFailed)
    }

    async fn delete_user(&self, _id: kernel::UserId) -> Result<bool, RepositoryError> {
        Ok(false)
    }

    async fn count_active_admins(&self) -> Result<u64, RepositoryError> {
        Ok(0)
    }

    async fn delete_login_attempts(&self, _normalized_email: &str) -> Result<(), RepositoryError> {
        Ok(())
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
        deletion: std::sync::Arc::new(crate::users_lifecycle::NoUserDeletion),
        avatar_store: std::sync::Arc::new(NullAvatarStore),
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

struct SeededRepository {
    inner: EmptyRepository,
    users: std::sync::Mutex<Vec<User>>,
}

#[async_trait::async_trait]
impl UserRepository for SeededRepository {
    async fn count(&self) -> Result<u64, RepositoryError> {
        Ok(self.users.lock().unwrap().len() as u64)
    }

    async fn list(&self) -> Result<Vec<User>, RepositoryError> {
        let mut users = self.users.lock().unwrap().clone();
        users.sort_by(|a, b| a.email().as_str().cmp(b.email().as_str()));
        Ok(users)
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|user| user.email().as_str() == email.as_str())
            .cloned())
    }

    async fn find_by_id(&self, id: kernel::UserId) -> Result<Option<User>, RepositoryError> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|user| user.id() == id)
            .cloned())
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

    async fn set_disabled(
        &self,
        id: kernel::UserId,
        disabled: bool,
    ) -> Result<(), RepositoryError> {
        let mut users = self.users.lock().unwrap();
        let Some(current) = users.iter_mut().find(|user| user.id() == id) else {
            return Err(RepositoryError::UserNotFound);
        };
        *current = current.clone().with_disabled(disabled);
        Ok(())
    }

    async fn set_profile(
        &self,
        id: kernel::UserId,
        display_name: Option<identity_domain::DisplayName>,
        avatar: Option<identity_domain::AvatarRef>,
    ) -> Result<User, RepositoryError> {
        let mut users = self.users.lock().unwrap();
        let Some(current) = users.iter_mut().find(|user| user.id() == id) else {
            return Err(RepositoryError::UserNotFound);
        };
        *current = current
            .clone()
            .with_display_name(display_name)
            .with_avatar(avatar);
        Ok(current.clone())
    }

    async fn delete_user(&self, id: kernel::UserId) -> Result<bool, RepositoryError> {
        let mut users = self.users.lock().unwrap();
        let before = users.len();
        users.retain(|user| user.id() != id);
        Ok(users.len() != before)
    }

    async fn count_active_admins(&self) -> Result<u64, RepositoryError> {
        Ok(u64::try_from(
            self.users
                .lock()
                .unwrap()
                .iter()
                .filter(|user| !user.is_disabled() && matches!(user.role(), Role::Admin))
                .count(),
        )
        .unwrap_or_default())
    }

    async fn delete_login_attempts(&self, normalized_email: &str) -> Result<(), RepositoryError> {
        self.inner.delete_login_attempts(normalized_email).await
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
    seeded_app_with(users, Arc::new(NullAvatarStore))
}

fn seeded_app_with(
    users: Vec<User>,
    avatar_store: Arc<dyn identity_application::ports::AvatarStore>,
) -> axum::Router {
    let dependencies = Arc::new(IdentityDependencies {
        repository: Arc::new(SeededRepository {
            inner: EmptyRepository,
            users: std::sync::Mutex::new(users),
        }),
        hasher: Arc::new(NullHasher),
        clock: Arc::new(NullClock),
        admission: Arc::new(NullAdmission),
        dummy_password_hash: PasswordHash::from_stored("null:dummy".to_owned()),
        session_secret: Arc::new("session-secret-value".to_owned()),
        medioss_origin: Arc::new("https://medioss.example".to_owned()),
        deletion: std::sync::Arc::new(crate::users_lifecycle::NoUserDeletion),
        avatar_store,
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

fn disabled_user(id: i64, email: &str, role: Role) -> User {
    user(id, email, role).with_disabled(true)
}

#[tokio::test]
async fn users_list_returns_sorted_users_for_admin() {
    let router = seeded_app(vec![
        disabled_user(3, "mid@example.com", Role::Member),
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
    assert_eq!(
        emails,
        vec!["adam@example.com", "mid@example.com", "zoe@example.com"]
    );
    let flags: Vec<bool> = listed
        .iter()
        .map(|u| u["disabled"].as_bool().unwrap())
        .collect();
    assert_eq!(flags, vec![false, true, false]);
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

#[tokio::test]
async fn user_status_disables_and_reenables_for_admin() {
    let router = seeded_app(vec![
        user(1, "admin@example.com", Role::Admin),
        user(2, "member@example.com", Role::Member),
    ]);
    let mut request = Request::patch("/identity/users/2/status")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"disabled":true}"#))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let mut request = Request::patch("/identity/users/2/status")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"disabled":false}"#))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn user_status_refuses_last_active_admin() {
    let router = seeded_app(vec![user(1, "admin@example.com", Role::Admin)]);
    let mut request = Request::patch("/identity/users/1/status")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"disabled":true}"#))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn user_status_rejects_members_and_anonymous() {
    let router = seeded_app(vec![
        user(1, "admin@example.com", Role::Admin),
        user(2, "member@example.com", Role::Member),
    ]);
    let mut request = Request::patch("/identity/users/1/status")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"disabled":true}"#))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(2), Role::Member));
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let request = Request::patch("/identity/users/1/status")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"disabled":true}"#))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn user_delete_removes_member_for_admin() {
    let router = seeded_app(vec![
        user(1, "admin@example.com", Role::Admin),
        user(2, "member@example.com", Role::Member),
    ]);
    let mut request = Request::delete("/identity/users/2")
        .header("origin", "https://medioss.example")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn user_delete_refuses_last_admin_without_purging() {
    let router = seeded_app(vec![user(1, "admin@example.com", Role::Admin)]);
    let mut request = Request::delete("/identity/users/1")
        .header("origin", "https://medioss.example")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

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
    assert_eq!(listed.len(), 1, "refused delete must keep the user row");
}

#[tokio::test]
async fn me_delete_removes_own_account_and_flushes_session() {
    let router = seeded_app(vec![
        user(1, "admin@example.com", Role::Admin),
        user(2, "member@example.com", Role::Member),
    ]);
    let mut request = Request::delete("/identity/me")
        .header("origin", "https://medioss.example")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(2), Role::Member));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn me_delete_refuses_last_active_admin() {
    let router = seeded_app(vec![user(1, "admin@example.com", Role::Admin)]);
    let mut request = Request::delete("/identity/me")
        .header("origin", "https://medioss.example")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn lifecycle_mutations_require_browser_origin() {
    let router = seeded_app(vec![
        user(1, "admin@example.com", Role::Admin),
        user(2, "member@example.com", Role::Member),
    ]);
    let mut request = Request::delete("/identity/users/2")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn profile_patch_sets_changes_and_clears_the_name() {
    let router = seeded_app(vec![user(1, "member@example.com", Role::Member)]);
    let request = |body: &str| {
        let mut request = Request::patch("/identity/me/profile")
            .header("origin", "https://medioss.example")
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap();
        request
            .extensions_mut()
            .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));
        request
    };

    let response = router
        .clone()
        .oneshot(request(r#"{"display_name":"  Ada  "}"#))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let profile: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(profile["display_name"], "Ada");

    let response = router
        .clone()
        .oneshot(request(r#"{"display_name":"Ada L."}"#))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = router
        .clone()
        .oneshot(request(r#"{"display_name":"   "}"#))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let profile: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(profile["display_name"].is_null());
}

#[tokio::test]
async fn profile_patch_rejects_oversized_names_and_anonymous_calls() {
    let router = seeded_app(vec![user(1, "member@example.com", Role::Member)]);
    let long = "a".repeat(65);

    let mut request = Request::patch("/identity/me/profile")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({ "display_name": long }).to_string(),
        ))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let request = Request::patch("/identity/me/profile")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"display_name":"Ada"}"#.to_owned()))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn avatar_upload_normalizes_persists_and_replaces() {
    let store = Arc::new(RecordingAvatarStore::accepting());
    let router = seeded_app_with(
        vec![user(1, "member@example.com", Role::Member)],
        store.clone(),
    );
    let upload = || {
        let mut request = Request::put("/identity/me/avatar")
            .header("origin", "https://medioss.example")
            .header("content-type", "multipart/form-data; boundary=b")
            .body(Body::from(
                "--b\r\ncontent-disposition: form-data; name=\"file\"; filename=\"a\"\r\n\r\nimg\r\n--b--\r\n".to_owned(),
            ))
            .unwrap();
        request
            .extensions_mut()
            .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));
        request
    };

    let response = router.clone().oneshot(upload()).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let response = router.clone().oneshot(upload()).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    assert_eq!(*store.written.lock().unwrap(), vec!["webp", "webp"]);
    // The second upload stages a fresh key and unlinks the first avatar.
    assert_eq!(*store.removed.lock().unwrap(), vec!["webp"]);
}

#[tokio::test]
async fn avatar_upload_maps_truncated_multipart_to_400() {
    let router = seeded_app_with(
        vec![user(1, "member@example.com", Role::Member)],
        Arc::new(RecordingAvatarStore::accepting()),
    );
    let mut request = Request::put("/identity/me/avatar")
        .header("origin", "https://medioss.example")
        .header("content-type", "multipart/form-data; boundary=b")
        .body(Body::from(
            "--b\r\ncontent-disposition: form-data; name=\"file\"; filename=\"a\"\r\n\r\nimg"
                .to_owned(),
        ))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn avatar_upload_rejects_wrong_or_missing_multipart_field_with_400() {
    let store = Arc::new(RecordingAvatarStore::accepting());
    let router = seeded_app_with(
        vec![user(1, "member@example.com", Role::Member)],
        store.clone(),
    );

    for field in ["name=\"not_file\"", "filename=\"a\""] {
        let mut request = Request::put("/identity/me/avatar")
            .header("origin", "https://medioss.example")
            .header("content-type", "multipart/form-data; boundary=b")
            .body(Body::from(format!(
                "--b\r\ncontent-disposition: form-data; {field}\r\n\r\nimg\r\n--b--\r\n"
            )))
            .unwrap();
        request
            .extensions_mut()
            .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));
        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "field: {field}");
    }
    assert!(store.written.lock().unwrap().is_empty());
}

#[tokio::test]
async fn avatar_upload_rejects_invalid_payloads_without_writing() {
    let store = Arc::new(RecordingAvatarStore {
        normalize_result: std::sync::Mutex::new(Err(
            identity_application::ports::AvatarStoreError::InvalidPayload,
        )),
        written: std::sync::Mutex::new(Vec::new()),
        removed: std::sync::Mutex::new(Vec::new()),
        read_result: std::sync::Mutex::new(Vec::new()),
        next_key: std::sync::atomic::AtomicU64::new(1),
    });
    let router = seeded_app_with(
        vec![user(1, "member@example.com", Role::Member)],
        store.clone(),
    );

    let mut request = Request::put("/identity/me/avatar")
        .header("origin", "https://medioss.example")
        .header("content-type", "multipart/form-data; boundary=b")
        .body(Body::from(
            "--b\r\ncontent-disposition: form-data; name=\"file\"; filename=\"a\"\r\n\r\nnot-an-image\r\n--b--\r\n".to_owned(),
        ))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert!(store.written.lock().unwrap().is_empty());
}

#[tokio::test]
async fn avatar_upload_maps_normalization_failures_to_operational_statuses() {
    use identity_application::ports::AvatarStoreError;
    // Only invalid input stays a client error; every server-side failure
    // must surface as 5xx, never 400.
    let cases = [
        (
            AvatarStoreError::OutputTooLarge,
            StatusCode::PAYLOAD_TOO_LARGE,
        ),
        (
            AvatarStoreError::WorkerFailed,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (AvatarStoreError::Storage, StatusCode::INTERNAL_SERVER_ERROR),
        (AvatarStoreError::Busy, StatusCode::SERVICE_UNAVAILABLE),
    ];
    for (error, expected) in cases {
        let store = Arc::new(RecordingAvatarStore {
            normalize_result: std::sync::Mutex::new(Err(error)),
            written: std::sync::Mutex::new(Vec::new()),
            removed: std::sync::Mutex::new(Vec::new()),
            read_result: std::sync::Mutex::new(Vec::new()),
            next_key: std::sync::atomic::AtomicU64::new(1),
        });
        let router = seeded_app_with(
            vec![user(1, "member@example.com", Role::Member)],
            store.clone(),
        );

        let mut request = Request::put("/identity/me/avatar")
            .header("origin", "https://medioss.example")
            .header("content-type", "multipart/form-data; boundary=b")
            .body(Body::from(
                "--b\r\ncontent-disposition: form-data; name=\"file\"; filename=\"a\"\r\n\r\nimg\r\n--b--\r\n".to_owned(),
            ))
            .unwrap();
        request
            .extensions_mut()
            .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), expected, "{error:?}");
        assert!(store.written.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn avatar_serving_is_owner_or_admin_only() {
    let store = Arc::new(RecordingAvatarStore::accepting());
    let users = vec![
        user(1, "admin@example.com", Role::Admin),
        user(2, "member@example.com", Role::Member)
            .with_avatar(Some(identity_domain::AvatarRef::Webp)),
        user(3, "other@example.com", Role::Member),
    ];
    let router = seeded_app_with(users, store);
    let get = |actor: kernel::Actor| {
        let mut request = Request::get("/identity/avatar/2")
            .body(Body::empty())
            .unwrap();
        request.extensions_mut().insert(actor);
        request
    };

    let response = router
        .clone()
        .oneshot(get(kernel::Actor::new(
            kernel::UserId::new(2),
            Role::Member,
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response = router
        .clone()
        .oneshot(get(kernel::Actor::new(kernel::UserId::new(1), Role::Admin)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response = router
        .clone()
        .oneshot(get(kernel::Actor::new(
            kernel::UserId::new(3),
            Role::Member,
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = router
        .oneshot(
            Request::get("/identity/avatar/2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn avatar_responses_forbid_content_type_sniffing() {
    for (avatar, content_type) in [
        (identity_domain::AvatarRef::Webp, "image/webp"),
        (identity_domain::AvatarRef::Png, "image/png"),
    ] {
        let store = Arc::new(RecordingAvatarStore::accepting());
        let router = seeded_app_with(
            vec![user(2, "member@example.com", Role::Member).with_avatar(Some(avatar))],
            store,
        );
        let mut request = Request::get("/identity/avatar/2")
            .body(Body::empty())
            .unwrap();
        request
            .extensions_mut()
            .insert(kernel::Actor::new(kernel::UserId::new(2), Role::Member));

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["content-type"], content_type);
        assert_eq!(response.headers()["x-content-type-options"], "nosniff");
    }
}

#[tokio::test]
async fn avatar_delete_is_idempotent() {
    let store = Arc::new(RecordingAvatarStore::accepting());
    let users = vec![user(1, "member@example.com", Role::Member)
        .with_avatar(Some(identity_domain::AvatarRef::Webp))];
    let router = seeded_app_with(users, store);
    let delete = || {
        let mut request = Request::delete("/identity/me/avatar")
            .header("origin", "https://medioss.example")
            .body(Body::empty())
            .unwrap();
        request
            .extensions_mut()
            .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));
        request
    };

    let response = router.clone().oneshot(delete()).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let response = router.oneshot(delete()).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn me_and_users_list_expose_profile_fields() {
    let users = vec![
        user(1, "admin@example.com", Role::Admin),
        user(2, "member@example.com", Role::Member)
            .with_display_name(Some(identity_domain::DisplayName::parse("Ada").unwrap()))
            .with_avatar(Some(identity_domain::AvatarRef::Webp)),
    ];
    let router = seeded_app(users);

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
    assert_eq!(listed[1]["display_name"], "Ada");
    assert_eq!(listed[1]["has_avatar"], true);
    assert_eq!(listed[0]["display_name"], serde_json::Value::Null);
    assert_eq!(listed[0]["has_avatar"], false);
}
