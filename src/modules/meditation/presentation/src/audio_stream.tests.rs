use axum::{
    body::Body,
    http::{header, StatusCode},
};

use super::sanitize_filename;
use crate::router_tests::*;

async fn fetch(
    app: &TestApp,
    path: &str,
    range: Option<&str>,
) -> (StatusCode, Vec<(axum::http::HeaderName, String)>, Vec<u8>) {
    let headers = range
        .map(|value| vec![(header::RANGE.as_str(), value)])
        .unwrap_or_default();
    call(
        app,
        Some(member_actor()),
        "GET",
        path,
        headers,
        Body::empty(),
    )
    .await
}

#[test]
fn sanitize_keeps_alphanumerics_and_dashes() {
    assert_eq!(sanitize_filename("Morning-Calm 2"), "Morning-Calm-2");
}

#[test]
fn sanitize_collapses_runs_and_trims_edges() {
    assert_eq!(sanitize_filename("  a / b..c  "), "a-b-c");
    assert_eq!(sanitize_filename("!!!"), "");
}

#[test]
fn sanitize_strips_header_injection_characters() {
    let cleaned = sanitize_filename("evil\"\r\nX-Injected: 1");
    assert!(!cleaned.contains(['"', '\r', '\n', ':', ' ']));
}

#[test]
fn sanitize_replaces_non_ascii_letters() {
    assert_eq!(sanitize_filename("Café"), "Caf");
}

#[tokio::test]
async fn audio_streams_full_body_with_accept_ranges() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, headers, body) = fetch(
        &app,
        &format!("/meditation/{}/audio", meditation.id()),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.len() as i64, meditation.size_bytes());
    assert_eq!(header_value(&headers, "accept-ranges"), Some("bytes"));
    assert_eq!(
        header_value(&headers, "content-type"),
        Some(meditation.content_type())
    );
    assert!(header_value(&headers, "content-disposition").is_none());
}

#[tokio::test]
async fn range_returns_206_with_content_range() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let path = format!("/meditation/{}/audio", meditation.id());
    let total = meditation.size_bytes();
    let (status, headers, body) = fetch(&app, &path, Some("bytes=2-5")).await;
    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(body.len(), 4);
    assert_eq!(
        header_value(&headers, "content-range"),
        Some(format!("bytes 2-5/{total}").as_str())
    );
}

#[tokio::test]
async fn unsatisfiable_range_is_416_with_total() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let path = format!("/meditation/{}/audio", meditation.id());
    let (status, headers, _) = fetch(&app, &path, Some("bytes=99999-")).await;
    assert_eq!(status, StatusCode::RANGE_NOT_SATISFIABLE);
    assert_eq!(
        header_value(&headers, "content-range"),
        Some(format!("bytes */{}", meditation.size_bytes()).as_str())
    );
}

#[tokio::test]
async fn download_is_an_attachment_named_after_the_meditation() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, headers, body) = fetch(
        &app,
        &format!("/meditation/{}/download", meditation.id()),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(!body.is_empty());
    assert_eq!(
        header_value(&headers, "content-disposition"),
        Some("attachment; filename=\"Seed.opus\"")
    );
}

#[tokio::test]
async fn unknown_meditation_is_404_for_audio_and_download() {
    let app = app();
    assert_eq!(
        fetch(&app, "/meditation/999/audio", None).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        fetch(&app, "/meditation/999/download", None).await.0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn anonymous_requests_are_401() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    for suffix in ["audio", "download"] {
        let (status, _, _) = call(
            &app,
            None,
            "GET",
            &format!("/meditation/{}/{suffix}", meditation.id()),
            vec![],
            Body::empty(),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{suffix}");
    }
}

#[tokio::test]
async fn missing_blob_is_an_opaque_500() {
    let app = app();
    let orphan = meditation_domain::Meditation::new(
        5,
        "Orphan",
        meditation_domain::STORED_CONTENT_TYPE,
        10,
        "no/such/key",
        1000,
        chrono::Utc::now(),
    )
    .unwrap();
    app.state.seed_meditation(orphan);
    let (status, _, body) = fetch(&app, "/meditation/5/audio", None).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&body).unwrap()["error"],
        "internal error"
    );
}
