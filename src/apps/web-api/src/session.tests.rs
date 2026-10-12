//! Session → Actor middleware tests (plan todo 11 acceptance).

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use crate::{
    router::router,
    state::{AppState, SystemClock, TokioSemaphoreAdmission},
};

async fn test_state() -> AppState {
    let directory = tempfile::tempdir().unwrap();
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(directory.path().join("test.sqlite"))
                .create_if_missing(true),
        )
        .await
        .unwrap();
    let connection = sea_orm::SqlxSqliteConnector::from_sqlx_sqlite_pool(pool.clone());
    identity_infrastructure::migrate(&connection).await.unwrap();
    meditation_infrastructure::migrations::migrate(&connection).await.unwrap();
    soundscape_infrastructure::migrations::migrate(&connection).await.unwrap();
    let store = tower_sessions_sqlx_store::SqliteStore::new(pool);
    store.migrate().await.unwrap();

    let config = crate::config::Config {
        data_dir: directory.path().to_path_buf(),
        port: 0,
        static_dir: directory.path().to_path_buf(),
        session_secret: crate::config::SessionSecret::new_for_tests("test-secret"),
        session_timeout: std::time::Duration::from_secs(3600),
        medioss_origin: "https://medioss.example".to_owned(),
        dev_origin: None,
        session_cookie_secure: false,
    };

    let origin = Arc::new(config.medioss_origin.clone());
    let meditation = crate::meditation::meditation_dependencies(
        connection.clone(),
        directory.path(),
        Arc::clone(&origin),
    );
    let soundscape = crate::soundscape::soundscape_dependencies(
        connection.clone(),
        directory.path(),
        origin,
    );
    AppState::new(
        Arc::new(config),
        Arc::new(store),
        meditation,
        soundscape,
        Arc::new(identity_infrastructure::SeaOrmUserRepository::new(connection)),
        identity_infrastructure::BoundedArgon2PasswordHasher::new(),
    )
}

const ORIGIN: &str = "https://medioss.example";

async fn setup_admin_via_api(app: &axum::Router) {
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/identity/setup")
                .header("origin", ORIGIN)
                .header("x-setup-secret", "test-secret")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"correct-horse"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED, "admin setup must succeed");
}

async fn login_and_capture_cookie(app: &axum::Router) -> Option<String> {
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/identity/login")
                .header("origin", ORIGIN)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"correct-horse"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK, "login must succeed");
    response
        .headers()
        .get("set-cookie")
        .and_then(|value| value.to_str().ok())
        .map(|cookie| cookie.split(';').next().unwrap_or("").to_owned())
}

#[tokio::test]
async fn admin_fallback_matches_only_the_first_path_segment() {
    let state = test_state().await;
    tokio::fs::create_dir_all(state.config.static_dir.join("admin/meditations"))
        .await
        .unwrap();
    tokio::fs::write(
        state.config.static_dir.join("admin/index.html"),
        "admin shell",
    )
    .await
    .unwrap();
    tokio::fs::write(
        state.config.static_dir.join("admin/meditations/index.html"),
        "meditations shell",
    )
    .await
    .unwrap();
    let app = router(state);

    let response = app
        .oneshot(
            Request::get("/admin/meditations-archive")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 4096).await.unwrap();
    assert_eq!(body.as_ref(), b"admin shell");
}

#[tokio::test]
async fn health_responds_without_a_session() {
    let state = test_state().await;
    let app = router(state);
    let response = app
        .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    if response.status() != StatusCode::OK {
        let body = axum::body::to_bytes(response.into_body(), 8192).await.unwrap();
        panic!("health body: {}", String::from_utf8_lossy(&body));
    }
}

#[tokio::test]
async fn me_without_a_session_returns_401_while_health_returns_200() {
    let state = test_state().await;
    let app = router(state);

    let me = app
        .clone()
        .oneshot(Request::get("/api/identity/me").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(me.status(), StatusCode::UNAUTHORIZED);

    let health = app
        .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);
}

#[tokio::test]
async fn middleware_inserts_actor_for_valid_session() {
    let state = test_state().await;
    let app = router(state);
    setup_admin_via_api(&app).await;
    let cookie = login_and_capture_cookie(&app).await.expect("login sets a cookie");

    let me = app
        .oneshot(
            Request::get("/api/identity/me")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(me.status(), StatusCode::OK);
    let body = axum::body::to_bytes(me.into_body(), 4096).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed["email"], "root@example.com");
    assert_eq!(parsed["role"], "admin");
}

#[tokio::test]
async fn stale_auth_version_session_inserts_no_actor() {
    let state = test_state().await;
    let app = router(state);
    setup_admin_via_api(&app).await;
    let cookie = login_and_capture_cookie(&app).await.expect("login sets a cookie");

    // Password change increments auth_version; the old session must stop working.
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/identity/me/password")
                .header("origin", ORIGIN)
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"fresh-permanent-pass"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK, "password change must succeed");

    // The OLD cookie (pre-change session) must no longer authenticate.
    let me = app
        .oneshot(
            Request::get("/api/identity/me")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(me.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn setup_endpoint_is_possession_gated() {
    let state = test_state().await;
    let app = router(state);

    let no_secret = app
        .clone()
        .oneshot(
            Request::post("/api/identity/setup")
                .header("origin", ORIGIN)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"correct-horse"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(no_secret.status(), StatusCode::FORBIDDEN);

    let wrong_secret = app
        .oneshot(
            Request::post("/api/identity/setup")
                .header("origin", ORIGIN)
                .header("x-setup-secret", "wrong")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"correct-horse"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(wrong_secret.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn setup_status_reports_required_then_completed() {
    let state = test_state().await;
    let app = router(state);

    let before = app
        .clone()
        .oneshot(Request::get("/api/identity/setup-status").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(before.status(), StatusCode::OK);
    let body = axum::body::to_bytes(before.into_body(), 4096).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed["setup_required"], true);

    setup_admin_via_api(&app).await;

    let after = app
        .oneshot(Request::get("/api/identity/setup-status").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(after.into_body(), 4096).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed["setup_required"], false);
}
