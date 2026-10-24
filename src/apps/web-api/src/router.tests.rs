use axum::{
    body::Body,
    http::{header, Request, Response, StatusCode},
    Router,
};
use tower::ServiceExt;

use crate::{
    router::router,
    test_support::{test_state, test_state_with_cookie_secure},
};

const ORIGIN: &str = "https://medioss.example";
const CREDENTIALS: &str = r#"{"email":"root@example.com","password":"correct-horse"}"#;

fn write_static(directory: &tempfile::TempDir, relative: &str, content: &str) {
    let path = directory.path().join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

async fn get(app: &Router, uri: &str) -> Response<Body> {
    app.clone()
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn body_text(response: Response<Body>) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn header_value<'a>(response: &'a Response<Body>, name: &str) -> &'a str {
    response.headers().get(name).unwrap().to_str().unwrap()
}

async fn post_credentials(app: &Router, uri: &str, setup_secret: Option<&str>) -> Response<Body> {
    let mut request = Request::post(uri)
        .header("origin", ORIGIN)
        .header("content-type", "application/json");
    if let Some(secret) = setup_secret {
        request = request.header("x-setup-secret", secret);
    }
    app.clone()
        .oneshot(request.body(Body::from(CREDENTIALS)).unwrap())
        .await
        .unwrap()
}

async fn login_set_cookie(app: &Router) -> String {
    let setup = post_credentials(app, "/api/identity/setup", Some("test-secret")).await;
    assert_eq!(setup.status(), StatusCode::CREATED);
    let login = post_credentials(app, "/api/identity/login", None).await;
    assert_eq!(login.status(), StatusCode::OK);
    header_value(&login, "set-cookie").to_owned()
}

#[tokio::test]
async fn health_reports_ok() {
    let (state, _directory) = test_state().await;
    let response = get(&router(state), "/api/health").await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&body_text(response).await).unwrap();
    assert_eq!(body, serde_json::json!({ "status": "ok" }));
}

#[tokio::test]
async fn meta_reports_name_version_source_and_license() {
    let (state, _directory) = test_state().await;
    let response = get(&router(state), "/api/meta").await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&body_text(response).await).unwrap();
    assert_eq!(body["name"], "MediOSS");
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(body["source"], "https://github.com/ahmet-cetinkaya/medioss");
    assert_eq!(body["license"], "AGPL-3.0-or-later");
}

#[tokio::test]
async fn unknown_api_route_returns_json_404() {
    let (state, _directory) = test_state().await;
    let response = get(&router(state), "/api/does-not-exist").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = serde_json::from_str(&body_text(response).await).unwrap();
    assert_eq!(body, serde_json::json!({ "error": "unknown api route" }));
}

#[tokio::test]
async fn static_files_are_served_from_the_static_dir() {
    let (state, directory) = test_state().await;
    write_static(&directory, "hello.txt", "hello static");
    let response = get(&router(state), "/hello.txt").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_text(response).await, "hello static");
}

#[tokio::test]
async fn missing_static_file_is_not_found() {
    let (state, _directory) = test_state().await;
    let response = get(&router(state), "/nope.txt").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn astro_assets_are_immutable_and_everything_else_revalidates() {
    let (state, directory) = test_state().await;
    write_static(&directory, "_astro/app.js", "console.log(1)");
    write_static(&directory, "plain.txt", "x");
    let app = router(state);

    let hashed = get(&app, "/_astro/app.js").await;
    let plain = get(&app, "/plain.txt").await;
    let api = get(&app, "/api/health").await;

    assert_eq!(
        header_value(&hashed, "cache-control"),
        "public, max-age=31536000, immutable"
    );
    assert_eq!(header_value(&plain, "cache-control"), "no-cache");
    assert_eq!(header_value(&api, "cache-control"), "no-cache");
}

#[tokio::test]
async fn responses_carry_nosniff() {
    let (state, directory) = test_state().await;
    write_static(&directory, "plain.txt", "x");
    let app = router(state);

    for uri in ["/api/health", "/api/unknown", "/plain.txt", "/missing"] {
        let response = get(&app, uri).await;
        assert_eq!(
            header_value(&response, "x-content-type-options"),
            "nosniff",
            "{uri}"
        );
    }
}

#[tokio::test]
async fn spa_fallbacks_serve_the_matching_index_page() {
    let (state, directory) = test_state().await;
    write_static(
        &directory,
        "admin/meditations/index.html",
        "admin-meditations",
    );
    write_static(
        &directory,
        "admin/soundscape/index.html",
        "admin-soundscape",
    );
    write_static(&directory, "admin/index.html", "admin-root");
    write_static(&directory, "player/index.html", "player-root");
    let app = router(state);

    let cases = [
        ("/admin/meditations/x", "admin-meditations"),
        ("/admin/soundscape/x", "admin-soundscape"),
        ("/admin/users", "admin-root"),
        ("/admin/meditationsfoo/x", "admin-root"),
        ("/player/x", "player-root"),
    ];
    for (uri, expected) in cases {
        let response = get(&app, uri).await;
        assert_eq!(response.status(), StatusCode::OK, "{uri}");
        assert_eq!(
            header_value(&response, "content-type"),
            "text/html; charset=utf-8",
            "{uri}"
        );
        assert_eq!(
            header_value(&response, "cache-control"),
            "no-cache",
            "{uri}"
        );
        assert_eq!(body_text(response).await, expected, "{uri}");
    }
}

#[tokio::test]
async fn spa_fallback_without_its_page_is_a_plain_404() {
    let (state, _directory) = test_state().await;
    let app = router(state);

    for uri in ["/admin/users", "/player/x"] {
        let response = get(&app, uri).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
        assert_eq!(body_text(response).await, "not found", "{uri}");
    }
}

#[tokio::test]
async fn plain_session_cookie_when_secure_is_disabled() {
    let (state, _directory) = test_state_with_cookie_secure(false).await;
    let cookie = login_set_cookie(&router(state)).await;

    assert!(cookie.starts_with("medioss-session="), "{cookie}");
    assert!(cookie.contains("HttpOnly"), "{cookie}");
    assert!(cookie.contains("SameSite=Strict"), "{cookie}");
    assert!(cookie.contains("Path=/"), "{cookie}");
    assert!(!cookie.contains("Secure"), "{cookie}");
}

#[tokio::test]
async fn host_prefixed_secure_session_cookie_when_secure_is_enabled() {
    let (state, _directory) = test_state_with_cookie_secure(true).await;
    let cookie = login_set_cookie(&router(state)).await;

    assert!(cookie.starts_with("__Host-medioss-session="), "{cookie}");
    assert!(cookie.contains("HttpOnly"), "{cookie}");
    assert!(cookie.contains("SameSite=Strict"), "{cookie}");
    assert!(cookie.contains("Path=/"), "{cookie}");
    assert!(cookie.contains("Secure"), "{cookie}");
    assert!(!cookie.contains("Domain"), "{cookie}");
}

#[tokio::test]
async fn static_response_without_session_sets_no_cookie() {
    let (state, _directory) = test_state().await;
    let response = get(&router(state), "/api/health").await;

    assert!(response.headers().get(header::SET_COOKIE).is_none());
}
