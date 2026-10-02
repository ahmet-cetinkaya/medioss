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
    commands::{delete_own_account, delete_user_as_admin, set_user_status},
    ports::{Clock, LoginAdmission, LoginThrottleRepository, PasswordHasher, UserRepository},
    ApplicationError,
};
use kernel::{Role, UserId};
use serde::Deserialize;
use tower_sessions::Session;

use crate::router::{ensure_browser_origin, ApiError, CurrentActor, SharedDependencies};

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
    CurrentActor(actor): CurrentActor,
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
    if actor.role() != Role::Admin {
        return Err(ApiError(ApplicationError::Forbidden));
    }
    set_user_status(
        dependencies.repository.as_ref(),
        &actor,
        UserId::new(id),
        request.disabled,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn user_delete<R, H, C, A>(
    State(dependencies): State<SharedDependencies<R, H, C, A>>,
    CurrentActor(actor): CurrentActor,
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
    if actor.role() != Role::Admin {
        return Err(ApiError(ApplicationError::Forbidden));
    }
    let target = UserId::new(id);
    // Purge module data first: if this fails, the user row stays intact.
    dependencies
        .deletion
        .on_user_deleted(target)
        .await
        .map_err(|_| ApiError(ApplicationError::Repository))?;
    match delete_user_as_admin(dependencies.repository.as_ref(), &actor, target).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(error) => Err(ApiError(error)),
    }
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
    dependencies
        .deletion
        .on_user_deleted(actor.user_id())
        .await
        .map_err(|_| ApiError(ApplicationError::Repository))?;
    delete_own_account(dependencies.repository.as_ref(), &actor).await?;
    session.flush().await?;
    Ok(StatusCode::NO_CONTENT)
}
