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
    let portability = web_api::portability::portability_dependencies(
        connection.clone(),
        Arc::new("http://localhost".to_owned()),
    );
    let deletion: Arc<dyn identity_presentation::UserDeletionListener> =
        Arc::new(web_api::user_deletion::CrossModuleUserDeletion::new(
            meditation.notes.clone(),
            meditation.playback.clone(),
            meditation.sessions.clone(),
            soundscape.mixes.clone(),
            mood_tracker.entries.clone(),
            std::path::PathBuf::from(&data_dir).join("media"),
        ));
    let state = AppState::new(
        Arc::new(config),
        Arc::new(store),
        meditation,
        mood_tracker,
        soundscape,
        portability,
        Arc::new(identity_infrastructure::SeaOrmUserRepository::new(
            connection,
        )),
        identity_infrastructure::BoundedArgon2PasswordHasher::new(),
        deletion,
        Arc::new(web_api::avatar_store::FileAvatarStore::new(
            std::path::PathBuf::from(data_dir).join("media"),
        )),
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

fn multipart_avatar(png: Vec<u8>) -> Vec<u8> {
    let boundary = "----mediossqa";
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        b"content-disposition: form-data; name=\"file\"; filename=\"a.png\"\r\ncontent-type: image/png\r\n\r\n",
    );
    body.extend_from_slice(&png);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

#[tokio::test]
async fn deleting_a_user_wipes_their_avatar_file() {
    let (state, guard) = test_state().await;
    let app = router(state);
    let origin = "http://localhost";

    let png = {
        let image = image::DynamicImage::new_rgb8(8, 8);
        let mut bytes = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .expect("encode png");
        bytes
    };

    let setup = app
        .clone()
        .oneshot(
            Request::post("/api/identity/setup")
                .header("origin", origin)
                .header("x-setup-secret", "test-setup-secret")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@local.test","password":"correct-horse-1"}"#.to_owned(),
                ))
                .expect("build"),
        )
        .await
        .expect("setup response");
    assert_eq!(setup.status(), StatusCode::CREATED);

    let login = app
        .clone()
        .oneshot(
            Request::post("/api/identity/login")
                .header("origin", origin)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@local.test","password":"correct-horse-1"}"#.to_owned(),
                ))
                .expect("build"),
        )
        .await
        .expect("login response");
    assert_eq!(login.status(), StatusCode::OK);
    let cookie = login
        .headers()
        .get("set-cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .expect("session cookie")
        .to_owned();

    let create = app
        .clone()
        .oneshot(
            Request::post("/api/identity/users")
                .header("origin", origin)
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"member@local.test","password":"member-horse-1"}"#.to_owned(),
                ))
                .expect("build"),
        )
        .await
        .expect("create response");
    assert_eq!(create.status(), StatusCode::CREATED);
    let member: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(create.into_body(), usize::MAX)
            .await
            .expect("create body"),
    )
    .expect("member json");
    let member_id = member["id"].as_i64().expect("member id");

    let member_login = app
        .clone()
        .oneshot(
            Request::post("/api/identity/login")
                .header("origin", origin)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"member@local.test","password":"member-horse-1"}"#.to_owned(),
                ))
                .expect("build"),
        )
        .await
        .expect("member login response");
    let member_cookie = member_login
        .headers()
        .get("set-cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .expect("member session cookie")
        .to_owned();

    let upload = app
        .clone()
        .oneshot(
            Request::put("/api/identity/me/avatar")
                .header("origin", origin)
                .header("cookie", &member_cookie)
                .header(
                    "content-type",
                    "multipart/form-data; boundary=----mediossqa",
                )
                .body(Body::from(multipart_avatar(png)))
                .expect("build"),
        )
        .await
        .expect("upload response");
    assert_eq!(upload.status(), StatusCode::CREATED);

    let member_media = guard.path().join("media").join(member_id.to_string());
    let uploaded = std::fs::read_dir(&member_media)
        .expect("member media directory")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert!(
        uploaded.len() == 1 && uploaded[0].starts_with("avatar-") && uploaded[0].ends_with(".webp"),
        "expected one staged avatar file, got {uploaded:?}"
    );

    let delete = app
        .clone()
        .oneshot(
            Request::delete(format!("/api/identity/users/{member_id}"))
                .header("origin", origin)
                .header("cookie", &cookie)
                .body(Body::empty())
                .expect("build"),
        )
        .await
        .expect("delete response");
    assert_eq!(delete.status(), StatusCode::NO_CONTENT);

    assert!(
        !member_media.exists(),
        "user media dir must be wiped on delete"
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
