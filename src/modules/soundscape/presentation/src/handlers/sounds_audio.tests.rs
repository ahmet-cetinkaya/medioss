use axum::{body::Body, http::StatusCode};

use crate::test_support::*;

async fn get_audio(
    range: Option<&str>,
) -> (StatusCode, Vec<(axum::http::HeaderName, String)>, Vec<u8>) {
    let app = app();
    let headers = range
        .map(|value| vec![("range", value)])
        .unwrap_or_default();
    call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds/1/audio",
        headers,
        Body::empty(),
    )
    .await
}

#[tokio::test]
async fn full_stream_returns_the_whole_blob_and_advertises_ranges() {
    let (status, headers, body) = get_audio(None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, b"rain-bytes");
    assert_eq!(
        header_value(&headers, "accept-ranges").as_deref(),
        Some("bytes")
    );
    assert_eq!(
        header_value(&headers, "content-length").as_deref(),
        Some("10")
    );
    assert_eq!(
        header_value(&headers, "content-type").as_deref(),
        Some("audio/ogg")
    );
    assert_eq!(header_value(&headers, "content-range"), None);
}

#[tokio::test]
async fn satisfiable_range_is_a_partial_content_response() {
    let (status, headers, _) = get_audio(Some("bytes=2-5")).await;
    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        header_value(&headers, "content-range").as_deref(),
        Some("bytes 2-5/10")
    );
    assert_eq!(
        header_value(&headers, "content-length").as_deref(),
        Some("4")
    );
}

#[tokio::test]
async fn unsatisfiable_range_is_416_with_the_total_size() {
    let (status, headers, _) = get_audio(Some("bytes=99-100")).await;
    assert_eq!(status, StatusCode::RANGE_NOT_SATISFIABLE);
    assert_eq!(
        header_value(&headers, "content-range").as_deref(),
        Some("bytes */10")
    );
}

#[tokio::test]
async fn malformed_range_header_falls_back_to_the_full_blob() {
    let (status, _, body) = get_audio(Some("items=0-1")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, b"rain-bytes");
}

#[tokio::test]
async fn unknown_sound_and_unreadable_blob_are_not_found() {
    let app = app();
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds/99/audio",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    *app.state.open_error.lock().unwrap() = Some("gone".into());
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds/1/audio",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn audio_requires_a_signed_in_actor() {
    let app = app();
    let (status, _, _) = call(
        &app,
        None,
        "GET",
        "/soundscape/sounds/1/audio",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn download_sanitises_the_filename_and_maps_the_extension() {
    let app = app();
    app.state.sounds.lock().unwrap()[0].name = "Rain / Storm!".into();
    let (status, headers, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds/1/download",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, b"rain-bytes");
    let disposition = header_value(&headers, "content-disposition").unwrap();
    assert!(
        disposition.starts_with("attachment; filename=\""),
        "{disposition}"
    );
    assert!(disposition.ends_with(".ogg\""), "{disposition}");
    assert!(
        !disposition.contains('/') && !disposition.contains('!'),
        "{disposition}"
    );
}
