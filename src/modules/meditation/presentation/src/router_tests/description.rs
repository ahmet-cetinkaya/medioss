use axum::{body::Body, http::StatusCode};

use meditation_application::{ports::ProgramRepository as _, test_support::FakeProgramRepository};

use super::helpers::*;

#[tokio::test]
async fn upload_with_description_roundtrips_through_library_and_detail() {
    let app = app();
    let program = FakeProgramRepository(app.state.clone())
        .insert("Holder".to_owned(), None, chrono::Utc::now())
        .await
        .unwrap()
        .id;
    let program_id = program.to_string();
    let (content_type, body) = multipart_body(&[
        ("name", "", b"Described"),
        ("program_id", "", program_id.as_bytes()),
        ("description", "", b"  A gentle start  "),
        ("file", "a.opus", b"OggS....OpusHead"),
    ]);
    let (status, _, body) = call(
        &app,
        Some(admin_actor()),
        "POST",
        "/meditation/audio",
        vec![("content-type", &content_type)],
        Body::from(body),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(created["description"], "A gentle start");
    let id = created["id"].as_i64().unwrap();

    let (_, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/{id}"),
        vec![],
        Body::empty(),
    )
    .await;
    let detail: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(detail["description"], "A gentle start");
    let listed = library_q(&app, "").await;
    assert_eq!(listed[0]["description"], "A gentle start");
}

#[tokio::test]
async fn meditation_without_description_serializes_null() {
    let app = app();
    let seeded = seed_meditation(&app).await;
    let (_, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/{}", seeded.id()),
        vec![],
        Body::empty(),
    )
    .await;
    let detail: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(detail["description"].is_null());
}

#[tokio::test]
async fn updating_the_description_sets_then_clears_it() {
    let app = app();
    let id = seed_meditation(&app).await.id();

    let (status, body) = put_description(
        &app,
        Some(admin_actor()),
        id,
        r#"{"description":"New text"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["description"], "New text");
    assert_eq!(body["id"], id);
    assert_eq!(body["name"], "Seed");

    for clear in [r#"{"description":null}"#, r#"{"description":""}"#] {
        put_description(&app, Some(admin_actor()), id, r#"{"description":"x"}"#).await;
        let (status, body) = put_description(&app, Some(admin_actor()), id, clear).await;
        assert_eq!(status, StatusCode::OK, "{clear}");
        assert!(body["description"].is_null(), "{clear}");
    }
}

#[tokio::test]
async fn too_long_description_is_400_with_the_exact_message() {
    let app = app();
    let id = seed_meditation(&app).await.id();
    let long = format!(r#"{{"description":"{}"}}"#, "a".repeat(2001));
    let (status, body) = put_description(&app, Some(admin_actor()), id, &long).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "description must be at most 2000 characters");

    let exact = format!(r#"{{"description":"{}"}}"#, "a".repeat(2000));
    let (status, _) = put_description(&app, Some(admin_actor()), id, &exact).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn description_update_covers_404_403_and_401() {
    let app = app();
    let id = seed_meditation(&app).await.id();
    let (status, _) =
        put_description(&app, Some(admin_actor()), 9999, r#"{"description":"x"}"#).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) =
        put_description(&app, Some(member_actor()), id, r#"{"description":"x"}"#).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = put_description(&app, None, id, r#"{"description":"x"}"#).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
