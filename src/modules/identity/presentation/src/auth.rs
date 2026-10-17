use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use identity_application::{
    commands::{change_password, setup_admin, Authenticate},
    ports::{Clock, LoginAdmission, LoginThrottleRepository, PasswordHasher, UserRepository},
    queries::is_setup_required,
};
use identity_domain::{Email, RawPassword};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use crate::actors::{CurrentActor, CurrentPrincipal};
use crate::api_error::ApiError;
use crate::dependencies::SharedDependencies;
use crate::origin_guard::{ensure_browser_origin, ensure_setup_secret};
use crate::session_keys::{SESSION_AUTH_VERSION_KEY, SESSION_USER_ID_KEY};
use crate::user_response::{role_to_str, user_response, UserResponse};

#[derive(Serialize)]
pub(crate) struct SetupStatusResponse {
    setup_required: bool,
}

pub(crate) async fn setup_status<R, H, C, A>(
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
pub(crate) struct CredentialsRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub(crate) struct ChangePasswordRequest {
    password: String,
}

pub(crate) async fn setup<R, H, C, A>(
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

pub(crate) async fn login<R, H, C, A>(
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

pub(crate) async fn logout<R, H, C, A>(
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

pub(crate) async fn me(
    CurrentPrincipal(principal): CurrentPrincipal,
) -> Result<Json<UserResponse>, ApiError> {
    Ok(Json(UserResponse {
        id: principal.id,
        email: principal.email,
        role: role_to_str(principal.role).to_owned(),
        must_change_password: principal.must_change_password,
        display_name: principal.display_name,
        has_avatar: principal.has_avatar,
    }))
}

pub(crate) async fn me_password<R, H, C, A>(
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
