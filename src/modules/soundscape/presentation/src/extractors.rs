use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use kernel::{Actor, Role};

pub(crate) struct CurrentActor(pub(crate) Actor);
impl<S: Send + Sync> FromRequestParts<S> for CurrentActor {
    type Rejection = StatusCode;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Actor>()
            .cloned()
            .map(Self)
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}
pub(crate) struct AdminActor(pub(crate) Actor);
impl<S: Send + Sync> FromRequestParts<S> for AdminActor {
    type Rejection = StatusCode;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let actor = parts
            .extensions
            .get::<Actor>()
            .cloned()
            .ok_or(StatusCode::UNAUTHORIZED)?;
        if actor.role() == Role::Admin {
            Ok(Self(actor))
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    }
}

pub(crate) fn admin(actor: &Actor) -> Result<(), StatusCode> {
    if actor.role() == Role::Admin {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

#[cfg(test)]
#[path = "extractors.tests.rs"]
mod tests;
