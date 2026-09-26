//! HTTP routing: JSON API under `/api`, everything else served from the
//! static directory (absent in dev until the frontend build lands).

use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use tower_http::services::ServeDir;
use tower_sessions::{Expiry, SessionManagerLayer};

use crate::{
    session::resolve_actor,
    state::{AppState, SystemClock, TokioSemaphoreAdmission},
};

pub fn router(state: AppState) -> Router {
    let static_files = ServeDir::new(&state.config.static_dir);

    let identity = identity_presentation::identity_router::<
        identity_infrastructure::SeaOrmUserRepository,
        identity_infrastructure::BoundedArgon2PasswordHasher,
        SystemClock,
        TokioSemaphoreAdmission,
    >()
    .with_state(state.identity_dependencies.clone());

    let api = Router::new()
        .route("/health", get(health))
        .merge(identity)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            resolve_actor,
        ))
        .fallback(unknown_api_route);

    Router::new()
        .nest("/api", api)
        .layer(session_layer(&state))
        .fallback_service(static_files)
        .with_state(state)
}

fn session_layer(state: &AppState) -> SessionManagerLayer<tower_sessions_sqlx_store::SqliteStore> {
    let secure = state.config.session_cookie_secure;
    let cookie_name = if secure {
        "__Host-medioss-session"
    } else {
        "medioss-session"
    };
    if !secure {
        tracing::warn!(
            "session cookie Secure is disabled (LAN mode): using plain cookie name {cookie_name}"
        );
    }
    let timeout = time::Duration::seconds(state.config.session_timeout.as_secs() as i64);
    SessionManagerLayer::new((*state.session_store).clone())
        .with_name(cookie_name)
        .with_path("/")
        .with_http_only(true)
        .with_same_site(tower_sessions::cookie::SameSite::Strict)
        .with_secure(secure)
        .with_expiry(Expiry::OnInactivity(timeout))
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}

async fn unknown_api_route() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "unknown api route" })),
    )
}
