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
