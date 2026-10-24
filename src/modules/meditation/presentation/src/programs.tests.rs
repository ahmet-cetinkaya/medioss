use axum::{body::Body, http::StatusCode};

use crate::router_tests::*;

async fn create(app: &TestApp, fields: &[(&str, &str, &[u8])]) -> (StatusCode, serde_json::Value) {
    let (content_type, body) = multipart_body(fields);
    let (status, _, body) = call(
        app,
        Some(admin_actor()),
        "POST",
        "/meditation/programs",
        vec![("content-type", &content_type)],
        Body::from(body),
    )
    .await;
    (status, serde_json::from_slice(&body).unwrap_or_default())
}

#[tokio::test]
async fn create_returns_201_with_program_shape() {
    let app = app();
    let (status, json) = create(
        &app,
        &[("title", "", b"Calm"), ("description", "", b"slow")],
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["title"], "Calm");
    assert_eq!(json["description"], "slow");
    assert_eq!(json["has_cover"], false);
    assert_eq!(json["category_id"], serde_json::Value::Null);
    assert_eq!(json["progress"], 0.0);
    assert!(json["created_at"].is_string());
}

#[tokio::test]
async fn create_without_title_is_400_and_blank_title_is_400() {
    let app = app();
    assert_eq!(
        create(&app, &[("description", "", b"x")]).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        create(&app, &[("title", "", b"   ")]).await.0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn oversized_title_field_is_413() {
    let app = app();
    let long = vec![b'a'; meditation_application::commands::MAX_NAME_LENGTH * 4 + 1];
    assert_eq!(
        create(&app, &[("title", "", &long)]).await.0,
        StatusCode::PAYLOAD_TOO_LARGE
    );
}

#[tokio::test]
async fn oversized_cover_is_413() {
    let app = app();
    let big = vec![0u8; meditation_application::commands::MAX_COVER_BYTES + 1];
    assert_eq!(
        create(&app, &[("title", "", b"P"), ("cover", "c.png", &big)])
            .await
            .0,
        StatusCode::PAYLOAD_TOO_LARGE
    );
}

#[tokio::test]
async fn create_with_cover_sets_has_cover() {
    let app = app();
    let png = [0x89, b'P', b'N', b'G', 0, 0, 0, 0, 0, 0, 0, 0];
    let (status, json) = create(&app, &[("title", "", b"P"), ("cover", "c.png", &png)]).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["has_cover"], true);
}

#[tokio::test]
async fn create_is_admin_only_and_checks_origin() {
    let app = app();
    let (content_type, body) = multipart_body(&[("title", "", b"P")]);
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "POST",
        "/meditation/programs",
        vec![("content-type", &content_type)],
        Body::from(body.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, _) = call(
        &app,
        None,
        "POST",
        "/meditation/programs",
        vec![("content-type", &content_type)],
        Body::from(body.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "POST",
        "/meditation/programs",
        vec![
            ("origin", "http://evil.test"),
            ("content-type", &content_type),
        ],
        Body::from(body),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn list_and_read_include_the_callers_progress() {
    let app = app();
    let program = seed_ordered_program(&app).await;
    for id in [1, 2] {
        let (status, _, _) = json_body(
            &app,
            Some(member_actor()),
            "PUT",
            &format!("/meditation/{id}/completed"),
            r#"{"completed":true}"#,
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }
    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/programs",
        "",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let listed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(listed[0]["id"], program);
    let progress = listed[0]["progress"].as_f64().unwrap();
    assert!((progress - 2.0 / 3.0).abs() < 1e-9, "{progress}");

    let (_, _, body) = json_body(
        &app,
        Some(admin_actor()),
        "GET",
        &format!("/meditation/programs/{program}"),
        "",
    )
    .await;
    let other: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(other["progress"], 0.0);
}

#[tokio::test]
async fn reads_require_a_session_and_unknown_program_is_404() {
    let app = app();
    let (status, _, _) = json_body(&app, None, "GET", "/meditation/programs", "").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _, _) = json_body(&app, None, "GET", "/meditation/programs/1", "").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/programs/999",
        "",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_answers_204_then_read_shows_new_values() {
    let app = app();
    let (_, created) = create(&app, &[("title", "", b"Old")]).await;
    let path = format!("/meditation/programs/{}", created["id"]);
    let (status, _, _) = json_body(
        &app,
        Some(admin_actor()),
        "PUT",
        &path,
        r#"{"title":"New","description":"d"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, _, body) = json_body(&app, Some(member_actor()), "GET", &path, "").await;
    let read: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(read["title"], "New");
    assert_eq!(read["description"], "d");
}

#[tokio::test]
async fn update_validates_scopes_and_finds() {
    let app = app();
    let (_, created) = create(&app, &[("title", "", b"P")]).await;
    let path = format!("/meditation/programs/{}", created["id"]);
    let (status, _, _) =
        json_body(&app, Some(admin_actor()), "PUT", &path, r#"{"title":"  "}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _, _) =
        json_body(&app, Some(member_actor()), "PUT", &path, r#"{"title":"x"}"#).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, _) = json_body(
        &app,
        Some(admin_actor()),
        "PUT",
        "/meditation/programs/999",
        r#"{"title":"x"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_answers_204_then_404() {
    let app = app();
    let (_, created) = create(&app, &[("title", "", b"P")]).await;
    let path = format!("/meditation/programs/{}", created["id"]);
    let (status, _, _) = json_body(&app, Some(member_actor()), "DELETE", &path, "").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, _) = json_body(&app, Some(admin_actor()), "DELETE", &path, "").await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _, _) = json_body(&app, Some(member_actor()), "GET", &path, "").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
