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
    commands::{change_password, create_user, setup_admin, Authenticate},
    ports::{Clock, LoginAdmission, LoginThrottleRepository, PasswordHasher, UserRepository},
    queries::{is_setup_required, list_users},
    ApplicationError,
};
use identity_domain::{Email, IdentityError, PasswordHash, RawPassword, User};
use kernel::{Actor, Role};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use crate::session_keys::{SESSION_AUTH_VERSION_KEY, SESSION_USER_ID_KEY};
use crate::users_lifecycle::{me_delete, user_delete, user_status};

/// Everything a handler needs, injected by web-api's composition root.
pub struct IdentityDependencies<R, H, C, A> {
    pub repository: Arc<R>,
    pub hasher: Arc<H>,
    pub clock: Arc<C>,
    pub admission: Arc<A>,
    pub dummy_password_hash: PasswordHash,
    pub session_secret: Arc<String>,
    pub medioss_origin: Arc<String>,
    pub deletion: Arc<dyn crate::users_lifecycle::UserDeletionListener>,
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
        .route("/identity/users", get(users_list))
        .route(
            "/identity/users/{id}/status",
            axum::routing::patch(user_status),
        )
        .route("/identity/users/{id}", axum::routing::delete(user_delete))
        .route("/identity/me", axum::routing::delete(me_delete))
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

#[derive(Deserialize)]
struct ChangePasswordRequest {
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
    let user = authenticate
        .execute(&request.email, &request.password)
        .await?;
    session.cycle_id().await?;
    session
        .insert(SESSION_USER_ID_KEY, user.id().value())
        .await?;
    session
        .insert(SESSION_AUTH_VERSION_KEY, user.auth_version())
        .await?;
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
    session.flush().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn me(CurrentPrincipal(principal): CurrentPrincipal) -> Result<Json<UserResponse>, ApiError> {
    Ok(Json(UserResponse {
        id: principal.id,
        email: principal.email,
        role: role_to_str(principal.role).to_owned(),
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

#[derive(Serialize)]
struct UserListItem {
    id: i64,
    email: String,
    role: String,
    must_change_password: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn users_list<R, H, C, A>(
    _actor: AdminActor,
    State(_dependencies): State<SharedDependencies<R, H, C, A>>,
) -> Result<Json<Vec<UserListItem>>, ApiError>
where
    R: UserRepository + LoginThrottleRepository + 'static,
    H: PasswordHasher + 'static,
    C: Clock + 'static,
    A: LoginAdmission + 'static,
{
    let users = list_users(_dependencies.repository.as_ref()).await?;
    Ok(Json(
        users
            .into_iter()
            .map(|user| UserListItem {
                id: user.id().value(),
                email: user.email().as_str().to_owned(),
                role: role_to_str(user.role()).to_owned(),
                must_change_password: user.must_change_password(),
                created_at: user.created_at(),
            })
            .collect(),
    ))
}

async fn me_password<R, H, C, A>(
    session: Session,
    State(dependencies): State<SharedDependencies<R, H, C, A>>,
    CurrentActor(actor): CurrentActor,
    headers: HeaderMap,
    Json(request): Json<ChangePasswordRequest>,
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
    session.cycle_id().await?;
    session
        .insert(SESSION_USER_ID_KEY, user.id().value())
        .await?;
    session
        .insert(SESSION_AUTH_VERSION_KEY, user.auth_version())
        .await?;
    Ok(Json(user_response(&user)))
}

fn user_response(user: &User) -> UserResponse {
    UserResponse {
        id: user.id().value(),
        email: user.email().as_str().to_owned(),
        role: role_to_str(user.role()).to_owned(),
        must_change_password: user.must_change_password(),
    }
}

pub(crate) fn ensure_browser_origin(headers: &HeaderMap, expected: &str) -> Result<(), ApiError> {
    match headers.get("origin").and_then(|value| value.to_str().ok()) {
        Some(origin) if origin.eq_ignore_ascii_case(expected) => Ok(()),
        // A missing or mismatched Origin is rejected: fail closed rather
        // than assume any Origin-less request is a same-origin browser one.
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

fn role_to_str(role: Role) -> &'static str {
    match role {
        Role::Admin => "admin",
        Role::Member => "member",
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

pub struct ApiError(pub ApplicationError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message, retry_after): (StatusCode, String, _) = match &self.0 {
            ApplicationError::InvalidInput(IdentityError::LoginThrottled { retry_after }) => (
                StatusCode::TOO_MANY_REQUESTS,
                "too many attempts".to_owned(),
                Some(retry_after.as_secs()),
            ),
            // Domain messages are user-facing by design and contain no secrets.
            ApplicationError::InvalidInput(error) => {
                (StatusCode::BAD_REQUEST, error.to_string(), None)
            }
            ApplicationError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "invalid credentials".to_owned(),
                None,
            ),
            ApplicationError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".to_owned(), None),
            ApplicationError::LastActiveAdmin => (
                StatusCode::BAD_REQUEST,
                "operation would leave the instance without an active admin".to_owned(),
                None,
            ),
            ApplicationError::SetupAlreadyCompleted => (
                StatusCode::CONFLICT,
                "setup already completed".to_owned(),
                None,
            ),
            ApplicationError::EmailAlreadyExists => (
                StatusCode::CONFLICT,
                "email already exists".to_owned(),
                None,
            ),
            ApplicationError::UserNotFound => (
                StatusCode::UNAUTHORIZED,
                "invalid credentials".to_owned(),
                None,
            ),
            ApplicationError::PasswordHasher | ApplicationError::Repository => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_owned(),
                None,
            ),
        };
        let mut response = (
            status,
            Json(serde_json::json!({
                "error": message,
                "retry_after": retry_after,
            })),
        )
            .into_response();
        if let Some(secs) = retry_after {
            // At least one second: sub-second penalties round up so the
            // client never sees retry_after = 0.
            let headers = response.headers_mut();
            headers.insert(
                "Retry-After",
                secs.max(1)
                    .to_string()
                    .parse()
                    .expect("retry-after is a valid header value"),
            );
        }
        response
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

/// Admin-only gate: same actor lookup as `CurrentActor`, plus a role check.
/// The extractor is the enforcement; handlers do not need the actor value.
#[allow(dead_code)]
struct AdminActor(Actor);

impl<S: Send + Sync> FromRequestParts<S> for AdminActor {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let actor = parts
            .extensions
            .get::<Actor>()
            .cloned()
            .ok_or(ApiError(ApplicationError::InvalidCredentials))?;
        if actor.role() == Role::Admin {
            Ok(Self(actor))
        } else {
            Err(ApiError(ApplicationError::Forbidden))
        }
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
