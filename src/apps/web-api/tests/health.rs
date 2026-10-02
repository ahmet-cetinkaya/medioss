//! Smoke tests over the real router: the health endpoint answers, unknown API
//! paths answer in JSON, and other unknown paths fall through to the static
//! handler as 404 when nothing is built yet.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
};
use tempfile::TempDir;
use tower::ServiceExt;
use web_api::{config, router::router, state::AppState};

/// Returns state pointing at an empty temp static dir; the guard must outlive
/// the request or the directory is removed mid-test.
async fn test_state() -> (AppState, TempDir) {
    let static_dir = tempfile::tempdir().expect("create temp static dir");

    let data_dir = static_dir.path().display().to_string();
    let config = config::from_vars(&|key| match key {
        "MEDIOSS_DATA_DIR" => Some(data_dir.clone()),
        "MEDIOSS_STATIC_DIR" => Some(data_dir.clone()),
        "SESSION_SECRET" => Some("test-setup-secret".to_owned()),
        "MEDIOSS_ORIGIN" => Some("http://localhost".to_owned()),
        _ => None,
    })
    .expect("valid test configuration");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(static_dir.path().join("health.sqlite"))
                .create_if_missing(true),
        )
        .await
        .expect("open sqlite");
    let connection = sea_orm::SqlxSqliteConnector::from_sqlx_sqlite_pool(pool.clone());
    identity_infrastructure::migrate(&connection)
        .await
        .expect("identity migrations");
    meditation_infrastructure::migrations::migrate(&connection)
        .await
        .expect("meditation migrations");
    soundscape_infrastructure::migrations::migrate(&connection)
        .await
        .expect("soundscape migrations");
    mood_tracker_infrastructure::migrations::migrate(&connection)
        .await
        .expect("mood tracker migrations");
    let store = tower_sessions_sqlx_store::SqliteStore::new(pool);
    store.migrate().await.expect("session store migrations");

    let meditation = web_api::meditation::meditation_dependencies(
        connection.clone(),
        static_dir.path(),
        Arc::new("http://localhost".to_owned()),
    );
    let soundscape = web_api::soundscape::soundscape_dependencies(
        connection.clone(),
        static_dir.path(),
        Arc::new("http://localhost".to_owned()),
    );
    let mood_tracker = web_api::mood_tracker::mood_tracker_dependencies(
        connection.clone(),
        Arc::new("http://localhost".to_owned()),
    );
    let deletion: Arc<dyn identity_presentation::UserDeletionListener> =
        Arc::new(web_api::user_deletion::CrossModuleUserDeletion::new(
            Arc::new(store.clone()),
            meditation.notes.clone(),
            meditation.playback.clone(),
            meditation.sessions.clone(),
            soundscape.mixes.clone(),
            mood_tracker.entries.clone(),
            std::path::PathBuf::from(data_dir).join("media"),
        ));
    let state = AppState::new(
        Arc::new(config),
        Arc::new(store),
        meditation,
        mood_tracker,
        soundscape,
        Arc::new(identity_infrastructure::SeaOrmUserRepository::new(
            connection,
        )),
        identity_infrastructure::BoundedArgon2PasswordHasher::new(),
        deletion,
    );

    (state, static_dir)
}

async fn get(uri: &str) -> Response<Body> {
    let (state, _static_dir) = test_state().await;

    router(state)
        .oneshot(
            Request::builder()
                .uri(uri)
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("router response")
}

fn assert_json_content_type(response: &Response<Body>) {
    let content_type = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .expect("content-type header")
        .to_str()
        .expect("content-type is ascii");
    assert!(
        content_type.starts_with("application/json"),
        "expected json content-type, got `{content_type}`"
    );
}

#[tokio::test]
async fn health_endpoint_returns_ok() {
    let response = get("/api/health").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_json_content_type(&response);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    assert_eq!(body.as_ref(), br#"{"status":"ok"}"#);
}

#[tokio::test]
async fn unknown_api_path_returns_json_not_found() {
    let response = get("/api/does-not-exist").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_json_content_type(&response);
}

#[tokio::test]
async fn unknown_path_returns_not_found() {
    let response = get("/nonexistent").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
