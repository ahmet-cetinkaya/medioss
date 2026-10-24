use axum::{body::Body, http::StatusCode};

use crate::router_tests::*;

async fn make_program(app: &TestApp, title: &str) -> i64 {
    let (content_type, body) = multipart_body(&[("title", "", title.as_bytes())]);
    let (_, _, body) = call(
        app,
        Some(admin_actor()),
        "POST",
        "/meditation/programs",
        vec![("content-type", &content_type)],
        Body::from(body),
    )
    .await;
    serde_json::from_slice::<serde_json::Value>(&body).unwrap()["id"]
        .as_i64()
        .unwrap()
}

async fn seed_meditation_named(app: &TestApp, id: i64) -> i64 {
    let meditation = meditation_domain::Meditation::new(
        id,
        "Second",
        meditation_domain::STORED_CONTENT_TYPE,
        10,
        "k/second",
        1000,
        chrono::Utc::now(),
    )
    .unwrap();
    app.state.seed_meditation(meditation);
    id
}

async fn add_member(app: &TestApp, program: i64, meditation: i64) {
    let (status, _, _) = json_body(
        app,
        Some(admin_actor()),
        "POST",
        &format!("/meditation/programs/{program}/members"),
        &serde_json::json!({ "meditation_id": meditation }).to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
}

async fn touch(app: &TestApp, meditation: i64) {
    let (status, _, _) = json_body(
        app,
        Some(member_actor()),
        "POST",
        &format!("/meditation/{meditation}/playback-state"),
        r#"{"position_ms":1000}"#,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn continue_is_204_before_anything_is_touched() {
    let app = app();
    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/continue",
        "",
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(body.is_empty());
}

#[tokio::test]
async fn continue_returns_program_and_meditation_after_playback() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let program = make_program(&app, "Calm").await;
    add_member(&app, program, meditation.id()).await;
    touch(&app, meditation.id()).await;

    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/continue",
        "",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["meditation_id"], meditation.id());
    assert_eq!(json["program"]["id"], program);
    assert_eq!(json["program"]["title"], "Calm");
    assert_eq!(json["program"]["progress"], 0.0);
}

#[tokio::test]
async fn continue_is_scoped_to_the_caller() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let program = make_program(&app, "Calm").await;
    add_member(&app, program, meditation.id()).await;
    touch(&app, meditation.id()).await;
    let (status, _, _) =
        json_body(&app, Some(admin_actor()), "GET", "/meditation/continue", "").await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn recent_programs_lists_touched_programs_with_timestamp() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let program = make_program(&app, "Calm").await;
    add_member(&app, program, meditation.id()).await;
    touch(&app, meditation.id()).await;

    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/recent-programs",
        "",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["program"]["id"], program);
    assert!(json[0]["last_touched_at"].is_string());
}

#[tokio::test]
async fn recent_programs_limit_caps_the_list() {
    let app = app();
    let first = seed_meditation(&app).await;
    let program_one = make_program(&app, "One").await;
    add_member(&app, program_one, first.id()).await;
    touch(&app, first.id()).await;
    let second = seed_meditation_named(&app, 2).await;
    let program_two = make_program(&app, "Two").await;
    add_member(&app, program_two, second).await;
    touch(&app, second).await;

    let (_, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/recent-programs",
        "",
    )
    .await;
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&body)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/recent-programs?limit=1",
        "",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["program"]["id"], program_two);
}

#[tokio::test]
async fn recent_programs_non_numeric_limit_is_a_client_error() {
    let app = app();
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/recent-programs?limit=many",
        "",
    )
    .await;
    assert!(status.is_client_error(), "got {status}");
}

#[tokio::test]
async fn both_surfaces_require_a_session() {
    let app = app();
    for path in ["/meditation/continue", "/meditation/recent-programs"] {
        let (status, _, _) = json_body(&app, None, "GET", path, "").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{path}");
    }
}
