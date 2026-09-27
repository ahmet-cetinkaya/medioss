//! Request extractors for authentication and authorization.

use axum::{extract::FromRequestParts, http::request::Parts};
use kernel::{Actor, Role};

use crate::response::ApiError;

/// Extracts the authenticated actor inserted by web-api middleware; every
/// meditation route requires it (spec FR-006: 401 otherwise).
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
            .ok_or(ApiError::unauthorized())
    }
}

/// Extracts an admin actor; content-management routes require it
/// (spec FR-005: 403 otherwise). The unauthorized-vs-forbidden order
/// matters: no session is 401, a member session is 403.
pub struct AdminActor(pub Actor);

impl<S> FromRequestParts<S> for AdminActor
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let actor = parts
            .extensions
            .get::<Actor>()
            .cloned()
            .ok_or(ApiError::unauthorized())?;
        if actor.role() != Role::Admin {
            return Err(ApiError::forbidden());
        }
        Ok(AdminActor(actor))
    }
}
