use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use kernel::Actor;

pub struct CurrentActor(pub Actor);

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
