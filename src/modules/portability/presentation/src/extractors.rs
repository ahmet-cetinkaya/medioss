use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use kernel::Actor;

pub struct CurrentActor(pub Actor);

impl<S> FromRequestParts<S> for CurrentActor
where
    S: Send + Sync,
{
    type Rejection = (axum::http::StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Actor>()
            .cloned()
            .map(CurrentActor)
            .ok_or((
                axum::http::StatusCode::UNAUTHORIZED,
                "authentication required",
            ))
    }
}
