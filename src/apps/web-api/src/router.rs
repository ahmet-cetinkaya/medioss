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

    let meditation =
        meditation_presentation::meditation_router().with_state(state.meditation.clone());

    let api = Router::new()
        .route("/health", get(health))
        .merge(identity)
        .merge(meditation)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            resolve_actor,
        ))
        .fallback(unknown_api_route);

    Router::new()
        .nest("/api", api)
        .layer(session_layer(&state))
        .fallback_service(static_files)
        .layer(axum::middleware::from_fn(cache_headers))
        .with_state(state)
}

/// Content-hashed assets are immutable; everything else must revalidate.
async fn cache_headers(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let immutable = request.uri().path().starts_with("/_astro/");
    let mut response = next.run(request).await;
    if !response.headers().contains_key("cache-control") {
        let value = if immutable {
            "public, max-age=31536000, immutable"
        } else {
            "no-cache"
        };
        if let Ok(parsed) = value.parse() {
            response.headers_mut().insert("cache-control", parsed);
        }
    }
    response
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
