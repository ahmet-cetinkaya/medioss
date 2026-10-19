//! HTTP routing: JSON API under `/api`, everything else served from the
//! static directory (absent in dev until the frontend build lands).

use axum::{
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
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
    let mood_tracker =
        mood_tracker_presentation::mood_tracker_router().with_state(state.mood_tracker.clone());
    let soundscape =
        soundscape_presentation::soundscape_router().with_state(state.soundscape.clone());
    let portability =
        portability_presentation::router::router().with_state(state.portability.clone());

    let api = Router::new()
        .route("/health", get(health))
        .route("/meta", get(meta))
        .merge(identity)
        .merge(meditation)
        .merge(mood_tracker)
        .merge(soundscape)
        .merge(portability)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            resolve_actor,
        ))
        .fallback(unknown_api_route);

    Router::new()
        .nest("/api", api)
        .layer(session_layer(&state))
        .route(
            "/admin/{*rest}",
            get({
                let static_dir = state.config.static_dir.clone();
                move |path: axum::extract::Path<String>| async move {
                    let page = if path.split('/').next() == Some("meditations") {
                        "admin/meditations/index.html"
                    } else if path.split('/').next() == Some("soundscape") {
                        "admin/soundscape/index.html"
                    } else {
                        "admin/index.html"
                    };
                    serve_static_file(&static_dir, page).await
                }
            }),
        )
        .route(
            "/player/{*rest}",
            get({
                let static_dir = state.config.static_dir.clone();
                move || async move { serve_static_file(&static_dir, "player/index.html").await }
            }),
        )
        .fallback_service(static_files)
        .layer(axum::middleware::from_fn(cache_headers))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
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

/// Instance facts for the About screen: app name and version. Version comes
/// from the single source of truth (the binary's Cargo version); public on
/// purpose — it carries no user data.
async fn meta() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "name": "MediOSS",
            "version": env!("CARGO_PKG_VERSION"),
            "source": "https://github.com/ahmet-cetinkaya/medioss",
            "license": "AGPL-3.0-or-later",
        })),
    )
}

async fn unknown_api_route() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "unknown api route" })),
    )
}

/// Serves one file from the static dir for the Admin SPA fallback.
async fn serve_static_file(
    static_dir: &std::path::Path,
    relative: &str,
) -> axum::response::Response {
    let full = static_dir.join(relative);
    match tokio::fs::read(&full).await {
        Ok(bytes) => {
            let content_type = match full.extension().and_then(|e| e.to_str()) {
                Some("html") => "text/html; charset=utf-8",
                Some("js") => "text/javascript; charset=utf-8",
                Some("css") => "text/css; charset=utf-8",
                Some("svg") => "image/svg+xml",
                _ => "application/octet-stream",
            };
            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, content_type)],
                bytes,
            )
                .into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}
