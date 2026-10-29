use axum::{body::Body, http::StatusCode};

use std::sync::Arc;

use meditation_application::test_support::{FakeBlobStore, RepoState};

use crate::meditation_router;
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

const SMALL_CAP: usize = 32;

/// An app whose upload cap is `SMALL_CAP`, so a short body can overflow it.
fn capped_app() -> TestApp {
    let state = RepoState::new();
    let store = FakeBlobStore::new();
    let mut deps = test_dependencies(state.clone(), &store);
    deps.max_upload_bytes = SMALL_CAP;
    TestApp {
        router: meditation_router(SMALL_CAP, TEST_MAX_COVER_BYTES).with_state(Arc::new(deps)),
        state,
        store,
    }
}

async fn raw_upload(app: &TestApp, body: Vec<u8>, content_type: &str) -> (StatusCode, Vec<u8>) {
    let (status, _, body) = call(
        app,
        Some(admin_actor()),
        "POST",
        "/meditation/audio",
        vec![("content-type", content_type)],
        Body::from(body),
    )
    .await;
    (status, body)
}

#[tokio::test]
async fn file_over_the_cap_is_413_with_no_body_and_deletes_the_staged_file() {
    let app = capped_app();
    let program = make_program(&app).await.to_string();
    let oversized = vec![b'O'; SMALL_CAP + 1];
    let (content_type, body) = multipart_body(&[
        ("name", "", b"Big"),
        ("program_id", "", program.as_bytes()),
        ("file", "a.ogg", &oversized),
    ]);

    let (status, response_body) = raw_upload(&app, body, &content_type).await;

    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert!(response_body.is_empty(), "the 413 body stays empty");
    assert_eq!(app.store.staging.file_count(), 0);
    assert_eq!(app.store.staging.deleted_count(), 1);
    assert!(app.store.is_empty());
}

#[tokio::test]
async fn truncated_stream_is_400_and_deletes_the_staged_file() {
    let app = capped_app();
    let (content_type, mut body) = multipart_body(&[("file", "a.ogg", b"OggS....OpusHead")]);
    // Cut the body inside the file field, before its closing boundary.
    let cut = body.len() - 40;
    body.truncate(cut);

    let (status, _) = raw_upload(&app, body, &content_type).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(app.store.staging.file_count(), 0);
    assert_eq!(app.store.staging.deleted_count(), 1);
}

#[tokio::test]
async fn insufficient_free_space_is_507_before_anything_is_staged() {
    let app = capped_app();
    app.store.staging.set_free_bytes(Some(SMALL_CAP as u64 - 1));
    let program = make_program(&app).await.to_string();
    let (content_type, body) = multipart_body(&[
        ("name", "", b"Calm"),
        ("program_id", "", program.as_bytes()),
        ("file", "a.ogg", OPUS_BYTES[..16].as_ref()),
    ]);

    let (status, response_body) = raw_upload(&app, body, &content_type).await;

    assert_eq!(status, StatusCode::INSUFFICIENT_STORAGE);
    let json: serde_json::Value = serde_json::from_slice(&response_body).unwrap();
    assert_eq!(json["error"], "insufficient storage for upload");
    assert_eq!(app.store.staging.file_count(), 0);
    assert!(app.store.is_empty());
}

#[tokio::test]
async fn unmeasurable_free_space_does_not_block_the_upload() {
    let app = capped_app();
    app.store.staging.set_free_bytes(None);
    let program = make_program(&app).await.to_string();
    let (content_type, body) = multipart_body(&[
        ("name", "", b"Calm"),
        ("program_id", "", program.as_bytes()),
        ("file", "a.ogg", OPUS_BYTES[..16].as_ref()),
    ]);

    let (status, _) = raw_upload(&app, body, &content_type).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(app.store.len(), 1);
    assert_eq!(app.store.staging.file_count(), 0, "moved into the store");
}

#[tokio::test]
async fn rejected_media_deletes_the_staged_file() {
    let app = app();
    let program = make_program(&app).await.to_string();

    let (status, _) = upload(
        &app,
        Some(admin_actor()),
        &[
            ("name", "", b"Junk"),
            ("program_id", "", program.as_bytes()),
            ("file", "a.txt", b"not audio at all"),
        ],
    )
    .await;

    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(app.store.staging.file_count(), 0);
    assert!(app.store.is_empty());
}

#[tokio::test]
async fn missing_name_deletes_the_staged_file() {
    let app = app();

    let (status, _) = upload(&app, Some(admin_actor()), &[("file", "a.ogg", OPUS_BYTES)]).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(app.store.staging.file_count(), 0);
}

#[tokio::test]
async fn a_repeated_file_field_keeps_the_last_one_and_discards_the_first() {
    let app = app();
    let program = make_program(&app).await.to_string();
    let (status, _) = upload(
        &app,
        Some(admin_actor()),
        &[
            ("name", "", b"Breath"),
            ("program_id", "", program.as_bytes()),
            ("file", "first.ogg", OPUS_BYTES),
            ("file", "second.ogg", OPUS_BYTES),
        ],
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(app.store.len(), 1, "only the last file is stored");
    assert_eq!(app.store.staging.file_count(), 0, "nothing stays staged");
    assert_eq!(
        app.store.staging.deleted_count(),
        1,
        "the first was discarded"
    );
}
