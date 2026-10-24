use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use identity_application::commands::authenticate::Authenticate;
use identity_application::commands::change_password::ChangePassword;
use identity_application::commands::setup_admin::SetupAdmin;
use identity_application::queries::setup_status::IsSetupRequired;
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

pub(crate) async fn setup_status(
    State(dependencies): State<SharedDependencies>,
) -> Result<Json<SetupStatusResponse>, ApiError> {
    let setup_required = dependencies.setup_status.handle(IsSetupRequired).await?;
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

pub(crate) async fn setup(
    State(dependencies): State<SharedDependencies>,
    headers: HeaderMap,
    Json(request): Json<CredentialsRequest>,
) -> Result<(StatusCode, Json<UserResponse>), ApiError> {
    ensure_browser_origin(&headers, &dependencies.medioss_origin)?;
    ensure_setup_secret(&headers, &dependencies.session_secret)?;
    let email = Email::parse(&request.email)?;
    let password = RawPassword::parse(&request.password)?;
    let user = dependencies
        .setup_admin
        .handle(SetupAdmin { email, password })
        .await?;
    Ok((StatusCode::CREATED, Json(user_response(&user))))
}

pub(crate) async fn login(
    session: Session,
    State(dependencies): State<SharedDependencies>,
    headers: HeaderMap,
    Json(request): Json<CredentialsRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    ensure_browser_origin(&headers, &dependencies.medioss_origin)?;
    let user = dependencies
        .authenticate
        .handle(Authenticate {
            identifier: request.email,
            password: request.password,
            bypass_throttle: false,
        })
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

pub(crate) async fn logout(
    CurrentActor(_actor): CurrentActor,
    State(dependencies): State<SharedDependencies>,
    session: Session,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
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

pub(crate) async fn me_password(
    session: Session,
    State(dependencies): State<SharedDependencies>,
    CurrentActor(actor): CurrentActor,
    headers: HeaderMap,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    ensure_browser_origin(&headers, &dependencies.medioss_origin)?;
    let new_password = RawPassword::parse(&request.password)?;
    let user = dependencies
        .change_password
        .handle(ChangePassword {
            user_id: actor.user_id(),
            new_password,
        })
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

#[cfg(test)]
#[path = "auth.tests.rs"]
mod tests;
