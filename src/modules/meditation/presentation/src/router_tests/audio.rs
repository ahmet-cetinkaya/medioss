use axum::{
    body::Body,
    http::{header, StatusCode},
};

use super::helpers::*;

#[tokio::test]
async fn range_matrix_over_stored_audio() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let path = format!("/meditation/{}/audio", meditation.id());

    // No Range: 200 with the full body.
    let (status, headers, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        &path,
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(!body.is_empty());
    assert!(headers
        .iter()
        .any(|(name, _)| name == header::ACCEPT_RANGES));

    // bytes=0-99: 206 with exactly 100 bytes (or the whole blob when smaller).
    let (status, headers, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        &path,
        vec![(header::RANGE.as_str(), "bytes=0-99")],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        body.len(),
        100.min(expected_len(&app, meditation.id()).await)
    );
    let content_range = headers
        .iter()
        .find(|(name, _)| name == header::CONTENT_RANGE)
        .expect("content-range present");
    assert!(
        content_range.1.starts_with("bytes 0-"),
        "clamped end for short blobs: {content_range:?}"
    );

    // Open-ended: bytes=10-
    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        &path,
        vec![(header::RANGE.as_str(), "bytes=10-")],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    let total = expected_len(&app, meditation.id()).await as usize;
    assert_eq!(body.len(), total - 10);

    // Suffix: bytes=-5 → the last 5 bytes.
    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        &path,
        vec![(header::RANGE.as_str(), "bytes=-5")],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(body.len(), 5);

    // Unsatisfiable: 416 with `bytes */total`.
    let (status, headers, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        &path,
        vec![(header::RANGE.as_str(), "bytes=999999-")],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::RANGE_NOT_SATISFIABLE);
    assert!(headers
        .iter()
        .any(|(name, value)| name == header::CONTENT_RANGE && value.starts_with("bytes */")));

    // Malformed: ignored, full 200.
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        &path,
        vec![(header::RANGE.as_str(), "not-a-range")],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn download_is_an_attachment_named_after_the_meditation() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, headers, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/{}/download", meditation.id()),
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let disposition = headers
        .iter()
        .find(|(name, _)| name == header::CONTENT_DISPOSITION)
        .expect("disposition present");
    assert!(disposition.1.starts_with("attachment; filename="));
    assert!(disposition.1.ends_with(".opus\""));
}
