//! HTTP surface for the identity module: DTOs, handlers, and routing.
//!
//! This crate never resolves sessions itself (web-api middleware inserts the
//! `Actor` into request extensions — ARCHITECTURE.md L335-340) and never
//! touches a database; all persistence flows through application ports.

use std::sync::Arc;

use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use identity_application::{
    commands::{Authenticate, change_password, create_user, setup_admin},
    ports::{Clock, LoginAdmission, LoginThrottleRepository, PasswordHasher, UserRepository},
    queries::is_setup_required,
    ApplicationError,
};
use identity_domain::{Email, IdentityError, PasswordHash, RawPassword, User};
use kernel::{Actor, Role};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use crate::session_keys::{SESSION_AUTH_VERSION_KEY, SESSION_USER_ID_KEY};

/// Everything a handler needs, injected by web-api's composition root.
pub struct IdentityDependencies<R, H, C, A> {
    pub repository: Arc<R>,
    pub hasher: Arc<H>,
    pub clock: Arc<C>,
    pub admission: Arc<A>,
    pub dummy_password_hash: PasswordHash,
    pub session_secret: Arc<String>,
    pub medioss_origin: Arc<String>,
}

pub type SharedDependencies<R, H, C, A> = Arc<IdentityDependencies<R, H, C, A>>;

pub fn identity_router<R, H, C, A>() -> Router<SharedDependencies<R, H, C, A>>
where
    R: UserRepository + LoginThrottleRepository + 'static,
    H: PasswordHasher + 'static,
    C: Clock + 'static,
    A: LoginAdmission + 'static,
{
    Router::new()
        .route("/identity/setup-status", get(setup_status))
        .route("/identity/setup", post(setup))
        .route("/identity/login", post(login))
        .route("/identity/logout", post(logout))
        .route("/identity/me", get(me))
        .route("/identity/users", post(users))
        .route("/identity/me/password", post(me_password))
}

#[derive(Serialize)]
struct SetupStatusResponse {
    setup_required: bool,
}

async fn setup_status<R, H, C, A>(
    State(dependencies): State<SharedDependencies<R, H, C, A>>,
) -> Result<Json<SetupStatusResponse>, ApiError>
where
    R: UserRepository,
    H: PasswordHasher,
    C: Clock,
    A: LoginAdmission,
{
    let setup_required = is_setup_required(dependencies.repository.as_ref()).await?;
    Ok(Json(SetupStatusResponse { setup_required }))
}

#[derive(Deserialize)]
struct CredentialsRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct UserResponse {
    id: i64,
    email: String,
    role: String,
    must_change_password: bool,
}

async fn setup<R, H, C, A>(
    State(dependencies): State<SharedDependencies<R, H, C, A>>,
    headers: HeaderMap,
    Json(request): Json<CredentialsRequest>,
) -> Result<(StatusCode, Json<UserResponse>), ApiError>
where
    R: UserRepository + LoginThrottleRepository,
    H: PasswordHasher,
    C: Clock,
    A: LoginAdmission,
{
    ensure_browser_origin(&headers, &dependencies.medioss_origin)?;
    ensure_setup_secret(&headers, &dependencies.session_secret)?;
    let email = Email::parse(&request.email)?;
    let password = RawPassword::parse(&request.password)?;
    let user = setup_admin(
        dependencies.repository.as_ref(),
        dependencies.hasher.as_ref(),
        dependencies.clock.as_ref(),
        email,
        &password,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(user_response(&user))))
}

async fn login<R, H, C, A>(
    session: Session,
    State(dependencies): State<SharedDependencies<R, H, C, A>>,
    headers: HeaderMap,
    Json(request): Json<CredentialsRequest>,
) -> Result<Json<UserResponse>, ApiError>
where
    R: UserRepository + LoginThrottleRepository,
    H: PasswordHasher,
    C: Clock,
    A: LoginAdmission,
{
    ensure_browser_origin(&headers, &dependencies.medioss_origin)?;
    let authenticate = Authenticate::new(
        dependencies.repository.as_ref(),
        dependencies.hasher.as_ref(),
        dependencies.clock.as_ref(),
        dependencies.admission.as_ref(),
        dependencies.dummy_password_hash.clone(),
    );
    let user = authenticate.execute(&request.email, &request.password).await?;
    let _ = session.cycle_id().await;
    session.insert(SESSION_USER_ID_KEY, user.id().value()).await?;
    session.insert(SESSION_AUTH_VERSION_KEY, user.auth_version()).await?;
    Ok(Json(user_response(&user)))
}

async fn logout<R, H, C, A>(
    CurrentActor(_actor): CurrentActor,
    State(dependencies): State<SharedDependencies<R, H, C, A>>,
    session: Session,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError>
where
    R: UserRepository + LoginThrottleRepository,
    H: PasswordHasher,
    C: Clock,
    A: LoginAdmission,
{
    ensure_browser_origin(&headers, &dependencies.medioss_origin)?;
    let _ = session.flush().await;
    Ok(StatusCode::NO_CONTENT)
}

async fn me(CurrentPrincipal(principal): CurrentPrincipal) -> Result<Json<UserResponse>, ApiError> {
    Ok(Json(UserResponse {
        id: principal.id,
        email: principal.email,
        role: match principal.role {
            Role::Admin => "admin",
            Role::Member => "member",
        }
        .to_owned(),
        must_change_password: principal.must_change_password,
    }))
}

async fn users<R, H, C, A>(
    State(dependencies): State<SharedDependencies<R, H, C, A>>,
    CurrentActor(actor): CurrentActor,
    headers: HeaderMap,
    Json(request): Json<CredentialsRequest>,
) -> Result<(StatusCode, Json<UserResponse>), ApiError>
where
    R: UserRepository + LoginThrottleRepository,
    H: PasswordHasher,
    C: Clock,
    A: LoginAdmission,
{
    ensure_browser_origin(&headers, &dependencies.medioss_origin)?;
    let email = Email::parse(&request.email)?;
    let temporary_password = RawPassword::parse(&request.password)?;
    let user = create_user(
        dependencies.repository.as_ref(),
        dependencies.hasher.as_ref(),
        dependencies.clock.as_ref(),
        &actor,
        email,
        &temporary_password,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(user_response(&user))))
}

async fn me_password<R, H, C, A>(
    session: Session,
    State(dependencies): State<SharedDependencies<R, H, C, A>>,
    CurrentActor(actor): CurrentActor,
    headers: HeaderMap,
    Json(request): Json<CredentialsRequest>,
) -> Result<Json<UserResponse>, ApiError>
where
    R: UserRepository + LoginThrottleRepository,
    H: PasswordHasher,
    C: Clock,
    A: LoginAdmission,
{
    ensure_browser_origin(&headers, &dependencies.medioss_origin)?;
    let new_password = RawPassword::parse(&request.password)?;
    let user = change_password(
        dependencies.repository.as_ref(),
        dependencies.hasher.as_ref(),
        &actor,
        &new_password,
    )
    .await?;
    let _ = session.cycle_id().await;
    session.insert(SESSION_USER_ID_KEY, user.id().value()).await?;
    session.insert(SESSION_AUTH_VERSION_KEY, user.auth_version()).await?;
    Ok(Json(user_response(&user)))
}

fn user_response(user: &User) -> UserResponse {
    UserResponse {
        id: user.id().value(),
        email: user.email().as_str().to_owned(),
        role: match user.role() {
            Role::Admin => "admin".to_owned(),
            Role::Member => "member".to_owned(),
        },
        must_change_password: user.must_change_password(),
    }
}

fn ensure_browser_origin(headers: &HeaderMap, expected: &str) -> Result<(), ApiError> {
    match headers
        .get("origin")
        .and_then(|value| value.to_str().ok())
    {
        Some(origin) if origin.eq_ignore_ascii_case(expected) => Ok(()),
        _ => Err(ApiError(ApplicationError::Forbidden)),
    }
}

fn ensure_setup_secret(headers: &HeaderMap, expected: &str) -> Result<(), ApiError> {
    let provided = headers
        .get("x-setup-secret")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
        Ok(())
    } else {
        Err(ApiError(ApplicationError::Forbidden))
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0u8, |acc, (l, r)| acc | (l ^ r))
        == 0
}

pub struct ApiError(ApplicationError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message, retry_after) = match &self.0 {
            ApplicationError::InvalidInput(IdentityError::LoginThrottled { retry_after }) => (
                StatusCode::TOO_MANY_REQUESTS,
                "too many attempts",
                Some(retry_after.as_secs()),
            ),
            ApplicationError::InvalidInput(IdentityError::WeakPassword) => {
                (StatusCode::BAD_REQUEST, "invalid input", None)
            }
            ApplicationError::InvalidInput(_) => (StatusCode::BAD_REQUEST, "invalid input", None),
            ApplicationError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "invalid credentials", None)
            }
            ApplicationError::Forbidden => (StatusCode::FORBIDDEN, "forbidden", None),
            ApplicationError::SetupAlreadyCompleted => {
                (StatusCode::CONFLICT, "setup already completed", None)
            }
            ApplicationError::EmailAlreadyExists => {
                (StatusCode::CONFLICT, "email already exists", None)
            }
            ApplicationError::UserNotFound => (StatusCode::UNAUTHORIZED, "invalid credentials", None),
            ApplicationError::PasswordHasher | ApplicationError::Repository => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error", None)
            }
        };
        let body = Json(serde_json::json!({
            "error": message,
            "retry_after": retry_after,
        }));
        (status, body).into_response()
    }
}

impl From<ApplicationError> for ApiError {
    fn from(error: ApplicationError) -> Self {
        Self(error)
    }
}

impl From<IdentityError> for ApiError {
    fn from(error: IdentityError) -> Self {
        Self(ApplicationError::InvalidInput(error))
    }
}

impl From<tower_sessions::session::Error> for ApiError {
    fn from(_: tower_sessions::session::Error) -> Self {
        Self(ApplicationError::Repository)
    }
}

#[cfg(test)]
impl ApiError {
    fn into_response_status(self) -> StatusCode {
        use axum::response::IntoResponse;
        let (parts, _) = self.into_response().into_parts();
        parts.status
    }
}

/// Reads the `Actor` inserted by web-api's session middleware; never resolves
/// a session (ARCHITECTURE.md L335-340).
pub struct CurrentActor(pub Actor);

impl<S> FromRequestParts<S> for CurrentActor
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Actor>()
            .cloned()
            .map(CurrentActor)
            .ok_or(ApiError(ApplicationError::InvalidCredentials))
    }
}

/// The authenticated principal snapshot carried inside the session by
/// web-api's middleware: enough to answer `GET /me` without a repository hit.
#[derive(Clone)]
pub struct Principal {
    pub id: i64,
    pub email: String,
    pub role: Role,
    pub must_change_password: bool,
    pub auth_version: u64,
}

pub struct CurrentPrincipal(pub Principal);

impl<S> FromRequestParts<S> for CurrentPrincipal
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Principal>()
            .cloned()
            .map(CurrentPrincipal)
            .ok_or(ApiError(ApplicationError::InvalidCredentials))
    }
}

#[cfg(test)]
#[path = "router.tests.rs"]
mod tests;
