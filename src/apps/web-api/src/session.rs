//! Session → Actor resolution middleware (ARCHITECTURE.md L332-345).
//!
//! Loads `{user_id, auth_version}` from the opaque server-side session,
//! reloads the user through the application port, and compares auth
//! versions; a mismatch or unknown user deletes the session and inserts no
//! Actor. Presentation crates only read what is inserted here.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use identity_presentation::Principal;
use tower_sessions::Session;

use crate::state::AppState;
use identity_presentation::session_keys::{SESSION_AUTH_VERSION_KEY, SESSION_USER_ID_KEY};

pub async fn resolve_actor(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let (mut parts, body) = request.into_parts();

    let Some(session) = parts.extensions.get::<Session>().cloned() else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let user_id: Option<i64> = session
        .get(SESSION_USER_ID_KEY)
        .await
        .unwrap_or(None);
    let auth_version: Option<u64> = session
        .get(SESSION_AUTH_VERSION_KEY)
        .await
        .unwrap_or(None);

    if let (Some(user_id), Some(auth_version)) = (user_id, auth_version) {
        let matched = state
            .identity
            .find_user_by_id(kernel::UserId::new(user_id))
            .await
            .ok()
            .flatten()
            .filter(|user| user.auth_version() == auth_version);

        match matched {
            Some(user) => {
                parts.extensions.insert(user.actor());
                parts.extensions.insert(Principal {
                    id: user.id().value(),
                    email: user.email().as_str().to_owned(),
                    role: user.role(),
                    must_change_password: user.must_change_password(),
                    auth_version: user.auth_version(),
                });
            }
            None => {
                let _ = session.flush().await;
            }
        }
    }

    next.run(Request::from_parts(parts, body)).await
}
