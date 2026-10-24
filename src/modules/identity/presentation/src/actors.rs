use axum::{extract::FromRequestParts, http::request::Parts};
use identity_application::ApplicationError;
use kernel::{Actor, Role};

use crate::api_error::ApiError;

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
pub struct AdminActor(pub Actor);

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
    pub display_name: Option<String>,
    pub has_avatar: bool,
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
#[path = "actors.tests.rs"]
mod tests;
