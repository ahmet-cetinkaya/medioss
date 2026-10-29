use axum::{body::Body, http::StatusCode};

use crate::test_support::*;

const MULTIPART: &str = "multipart/form-data; boundary=XB";

fn part(name: &str, value: &str) -> String {
    format!("--XB\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n")
}

fn file_part(bytes: &str) -> String {
    format!("--XB\r\nContent-Disposition: form-data; name=\"file\"; filename=\"a.ogg\"\r\n\r\n{bytes}\r\n")
}

async fn upload(parts: &[String]) -> (StatusCode, Vec<u8>) {
    let app = app();
    let body = format!("{}--XB--\r\n", parts.concat());
    let (status, _, body) = call(
        &app,
        Some(admin_actor()),
        "POST",
        "/soundscape/sounds",
        vec![("content-type", MULTIPART)],
        Body::from(body),
    )
    .await;
    (status, body)
}

#[tokio::test]
async fn list_serialises_sounds_with_icon_flags() {
    let app = app();
    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let sounds: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(sounds[0]["name"], "Rain");
    assert_eq!(sounds[0]["category_id"], 1);
    assert_eq!(sounds[0]["has_icon"], false);
    assert_eq!(sounds[0]["builtin"], false);
}

#[tokio::test]
async fn upload_creates_a_sound_with_the_converted_audio() {
    let (status, body) = upload(&[
        part("name", "Wind"),
        part("category_id", "1"),
        file_part("audio"),
    ])
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "{}",
        String::from_utf8_lossy(&body)
    );
    let sound: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(sound["name"], "Wind");
    assert_eq!(sound["content_type"], "audio/ogg");
    assert_eq!(sound["duration_ms"], 60_000);
}

#[tokio::test]
async fn upload_missing_required_parts_is_a_bad_request() {
    let (status, _) = upload(&[part("category_id", "1"), file_part("audio")]).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = upload(&[part("name", "Wind"), file_part("audio")]).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = upload(&[part("name", "Wind"), part("category_id", "1")]).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn upload_with_a_non_numeric_category_is_a_bad_request() {
    let (status, _) = upload(&[
        part("name", "Wind"),
        part("category_id", "abc"),
        file_part("audio"),
    ])
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn upload_into_an_unknown_category_is_rejected() {
    let (status, _) = upload(&[
        part("name", "Wind"),
        part("category_id", "99"),
        file_part("audio"),
    ])
    .await;
    assert!(status.is_client_error(), "got {status}");
}

#[tokio::test]
async fn members_cannot_upload() {
    let app = app();
    let body = format!("{}--XB--\r\n", part("name", "Wind"));
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "POST",
        "/soundscape/sounds",
        vec![("content-type", MULTIPART)],
        Body::from(body),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

const SMALL_CAP: usize = 32;

async fn raw_upload(app: &TestApp, body: String) -> (StatusCode, Vec<u8>) {
    let (status, _, body) = call(
        app,
        Some(admin_actor()),
        "POST",
        "/soundscape/sounds",
        vec![("content-type", MULTIPART)],
        Body::from(body),
    )
    .await;
    (status, body)
}

fn complete_upload(file: &str) -> String {
    format!(
        "{}--XB--\r\n",
        [
            part("name", "Wind"),
            part("category_id", "1"),
            file_part(file)
        ]
        .concat()
    )
}

#[tokio::test]
async fn successful_upload_moves_the_staged_file_out_of_staging() {
    let app = app();

    let (status, _) = raw_upload(&app, complete_upload("audio")).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(app.state.staging.file_count(), 0);
    assert_eq!(app.state.staging.deleted_count(), 0);
}

#[tokio::test]
async fn file_over_the_cap_is_413_with_no_body_and_deletes_the_staged_file() {
    let app = app_with_cap(SMALL_CAP);

    let (status, body) = raw_upload(&app, complete_upload(&"O".repeat(SMALL_CAP + 1))).await;

    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert!(body.is_empty(), "the 413 body stays empty");
    assert_eq!(app.state.staging.file_count(), 0);
    assert_eq!(app.state.staging.deleted_count(), 1);
    assert!(app.state.store.lock().unwrap().is_empty());
}

#[tokio::test]
async fn staging_volume_smaller_than_the_cap_is_507_before_any_byte_is_staged() {
    let app = app_with_cap(SMALL_CAP);
    app.state.staging.set_free_bytes(Some(SMALL_CAP as u64 - 1));

    let (status, _) = raw_upload(&app, complete_upload("audio")).await;

    assert_eq!(status, StatusCode::INSUFFICIENT_STORAGE);
    assert_eq!(app.state.staging.file_count(), 0);
}

#[tokio::test]
async fn truncated_stream_is_400_and_deletes_the_staged_file() {
    let app = app_with_cap(SMALL_CAP);
    let mut body = complete_upload("OggS....OpusHead");
    // Cut the body inside the file part, before its closing boundary.
    body.truncate(body.len() - 20);

    let (status, _) = raw_upload(&app, body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(app.state.staging.file_count(), 0);
}

#[tokio::test]
async fn later_validation_failures_delete_the_staged_file() {
    let missing_name = format!(
        "{}--XB--\r\n",
        [part("category_id", "1"), file_part("audio")].concat()
    );
    let unknown_category = format!(
        "{}--XB--\r\n",
        [
            part("name", "Wind"),
            part("category_id", "99"),
            file_part("audio")
        ]
        .concat()
    );
    let bad_icon_key = format!(
        "{}--XB--\r\n",
        [
            part("name", "Wind"),
            part("category_id", "1"),
            part("icon_key", "Bad Key!"),
            file_part("audio")
        ]
        .concat()
    );
    let non_numeric_category = format!(
        "{}--XB--\r\n",
        [
            part("name", "Wind"),
            file_part("audio"),
            part("category_id", "abc")
        ]
        .concat()
    );

    for body in [
        missing_name,
        unknown_category,
        bad_icon_key,
        non_numeric_category,
    ] {
        let app = app();

        let (status, _) = raw_upload(&app, body).await;

        assert!(status.is_client_error(), "got {status}");
        assert_eq!(app.state.staging.file_count(), 0);
    }
}

#[tokio::test]
async fn a_repeated_file_field_keeps_the_last_one_and_discards_the_first() {
    let app = app();
    let body = format!(
        "{}--XB--\r\n",
        [
            part("name", "Wind"),
            part("category_id", "1"),
            file_part("first"),
            file_part("second"),
        ]
        .concat()
    );

    let (status, _) = raw_upload(&app, body).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        app.state.store.lock().unwrap().len(),
        1,
        "only the last file is stored"
    );
    assert_eq!(app.state.staging.file_count(), 0, "nothing stays staged");
    assert_eq!(
        app.state.staging.deleted_count(),
        1,
        "the first was discarded"
    );
}
