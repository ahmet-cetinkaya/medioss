use axum::{body::Body, http::StatusCode};

use crate::router_tests::*;

const OPUS_BYTES: &[u8] = b"OggS....OpusHead-payload-for-tests";

async fn upload(
    app: &TestApp,
    actor: Option<kernel::Actor>,
    fields: &[(&str, &str, &[u8])],
) -> (StatusCode, serde_json::Value) {
    let (content_type, body) = multipart_body(fields);
    let (status, _, body) = call(
        app,
        actor,
        "POST",
        "/meditation/audio",
        vec![("content-type", &content_type)],
        Body::from(body),
    )
    .await;
    (status, serde_json::from_slice(&body).unwrap_or_default())
}

async fn make_program(app: &TestApp) -> i64 {
    let (content_type, body) = multipart_body(&[("title", "", b"P")]);
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

#[tokio::test]
async fn valid_upload_returns_201_with_meditation_shape() {
    let app = app();
    let program = make_program(&app).await.to_string();
    let (status, json) = upload(
        &app,
        Some(admin_actor()),
        &[
            ("name", "", b"Breath"),
            ("program_id", "", program.as_bytes()),
            ("description", "", b"slow"),
            ("file", "a.ogg", OPUS_BYTES),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["name"], "Breath");
    assert_eq!(json["description"], "slow");
    assert_eq!(json["completed"], false);
    assert_eq!(json["progress_percent"], 0);
    assert!(json["id"].is_i64());
    assert_eq!(app.store.len(), 1);
}

#[tokio::test]
async fn members_and_anonymous_callers_are_rejected_before_parsing() {
    let app = app();
    let fields: &[(&str, &str, &[u8])] = &[("name", "", b"x")];
    assert_eq!(
        upload(&app, Some(member_actor()), fields).await.0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(upload(&app, None, fields).await.0, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn missing_required_fields_are_400() {
    let app = app();
    let program = make_program(&app).await.to_string();
    let full: [(&str, &str, &[u8]); 3] = [
        ("name", "", b"Breath"),
        ("program_id", "", program.as_bytes()),
        ("file", "a.ogg", OPUS_BYTES),
    ];
    for skipped in 0..full.len() {
        let fields: Vec<_> = full
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != skipped)
            .map(|(_, field)| *field)
            .collect();
        let (status, _) = upload(&app, Some(admin_actor()), &fields).await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "without {}",
            full[skipped].0
        );
    }
}

#[tokio::test]
async fn non_numeric_program_id_is_400() {
    let app = app();
    let (status, _) = upload(
        &app,
        Some(admin_actor()),
        &[
            ("name", "", b"x"),
            ("program_id", "", b"abc"),
            ("file", "a.ogg", OPUS_BYTES),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn oversized_program_id_field_is_413() {
    let app = app();
    let long = vec![b'1'; 64];
    let (status, _) = upload(&app, Some(admin_actor()), &[("program_id", "", &long)]).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn oversized_name_field_is_413() {
    let app = app();
    let long = vec![b'a'; meditation_application::commands::MAX_NAME_LENGTH * 4 + 1];
    let (status, _) = upload(&app, Some(admin_actor()), &[("name", "", &long)]).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn non_audio_bytes_are_rejected_as_client_error() {
    let app = app();
    let program = make_program(&app).await.to_string();
    let (status, _) = upload(
        &app,
        Some(admin_actor()),
        &[
            ("name", "", b"x"),
            ("program_id", "", program.as_bytes()),
            ("file", "a.txt", b"definitely not audio"),
        ],
    )
    .await;
    assert!(status.is_client_error(), "got {status}");
    assert_eq!(app.store.len(), 0);
}

#[tokio::test]
async fn unknown_program_is_404() {
    let app = app();
    let (status, _) = upload(
        &app,
        Some(admin_actor()),
        &[
            ("name", "", b"x"),
            ("program_id", "", b"999"),
            ("file", "a.ogg", OPUS_BYTES),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn unknown_fields_are_ignored() {
    let app = app();
    let program = make_program(&app).await.to_string();
    let (status, _) = upload(
        &app,
        Some(admin_actor()),
        &[
            ("surprise", "", b"ignored"),
            ("name", "", b"x"),
            ("program_id", "", program.as_bytes()),
            ("file", "a.ogg", OPUS_BYTES),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn foreign_origin_is_403() {
    let app = app();
    let (content_type, body) = multipart_body(&[("name", "", b"x")]);
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "POST",
        "/meditation/audio",
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
async fn non_multipart_body_is_a_client_error() {
    let app = app();
    let (status, _, _) =
        json_body(&app, Some(admin_actor()), "POST", "/meditation/audio", "{}").await;
    assert!(status.is_client_error(), "got {status}");
}
