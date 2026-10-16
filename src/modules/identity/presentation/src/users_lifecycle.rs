//! Account lifecycle routes: admin disable/re-enable, admin delete,
//! member self-delete. Cross-module data purge runs through the
//! `UserDeletionListener` port, implemented by the composition root
//! (web-api) and injected into `IdentityDependencies`.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use identity_application::{
    commands::{
        delete_own_account, delete_user_as_admin, ensure_admin_can_delete_user,
        ensure_can_delete_own_account, set_user_status,
    },
    ports::{Clock, LoginAdmission, LoginThrottleRepository, PasswordHasher, UserRepository},
    ApplicationError,
};
use kernel::UserId;
use serde::Deserialize;
use tower_sessions::Session;

use crate::actors::{AdminActor, CurrentActor};
use crate::api_error::ApiError;
use crate::dependencies::SharedDependencies;
use crate::origin_guard::ensure_browser_origin;

/// Purges other modules' data and media for a user. The composition root
/// implements this; it runs only after all identity guards pass, so a
/// failure here aborts with 500 and nothing is deleted on the identity
/// side.
#[async_trait::async_trait]
pub trait UserDeletionListener: Send + Sync {
    async fn on_user_deleted(&self, user_id: UserId) -> Result<(), String>;
}

/// No-op listener for tests and non-orchestrated deployments.
pub struct NoUserDeletion;

#[async_trait::async_trait]
impl UserDeletionListener for NoUserDeletion {
    async fn on_user_deleted(&self, _user_id: UserId) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct UserStatusRequest {
    pub disabled: bool,
}

pub async fn user_status<R, H, C, A>(
    State(dependencies): State<SharedDependencies<R, H, C, A>>,
    actor: AdminActor,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(request): Json<UserStatusRequest>,
) -> Result<StatusCode, ApiError>
where
    R: UserRepository + LoginThrottleRepository + 'static,
    H: PasswordHasher + 'static,
    C: Clock + 'static,
    A: LoginAdmission + 'static,
{
    ensure_browser_origin(&headers, &dependencies.medioss_origin)?;
    set_user_status(
        dependencies.repository.as_ref(),
        &actor.0,
        UserId::new(id),
        request.disabled,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn user_delete<R, H, C, A>(
    State(dependencies): State<SharedDependencies<R, H, C, A>>,
    actor: AdminActor,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError>
where
    R: UserRepository + LoginThrottleRepository + 'static,
    H: PasswordHasher + 'static,
    C: Clock + 'static,
    A: LoginAdmission + 'static,
{
    ensure_browser_origin(&headers, &dependencies.medioss_origin)?;
    let actor = actor.0;
    let target = UserId::new(id);
    // Guards first: a refused request (last admin, unknown target) must
    // never destroy the target's data.
    ensure_admin_can_delete_user(dependencies.repository.as_ref(), &actor, target).await?;
    dependencies
        .deletion
        .on_user_deleted(target)
        .await
        .map_err(|_| ApiError::from(ApplicationError::Repository))?;
    delete_user_as_admin(dependencies.repository.as_ref(), &actor, target).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn me_delete<R, H, C, A>(
    session: Session,
    State(dependencies): State<SharedDependencies<R, H, C, A>>,
    CurrentActor(actor): CurrentActor,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError>
where
    R: UserRepository + LoginThrottleRepository + 'static,
    H: PasswordHasher + 'static,
    C: Clock + 'static,
    A: LoginAdmission + 'static,
{
    ensure_browser_origin(&headers, &dependencies.medioss_origin)?;
    // Guards first: a refused self-deletion (last admin) must never
    // destroy the caller's data.
    ensure_can_delete_own_account(dependencies.repository.as_ref(), &actor).await?;
    dependencies
        .deletion
        .on_user_deleted(actor.user_id())
        .await
        .map_err(|_| ApiError::from(ApplicationError::Repository))?;
    delete_own_account(dependencies.repository.as_ref(), &actor).await?;
    session.flush().await?;
    Ok(StatusCode::NO_CONTENT)
}
