use axum::{body::Body, http::header, http::StatusCode};

use super::helpers::*;

#[tokio::test]
async fn download_is_an_attachment_named_after_the_sound() {
    let app = app();
    let (status, headers, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds/1/download",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let disposition = headers
        .iter()
        .find(|(name, _)| name == header::CONTENT_DISPOSITION)
        .expect("disposition present");
    assert!(disposition.1.starts_with("attachment; filename="));
    assert!(disposition.1.ends_with(".ogg\""));
    assert_eq!(body, b"rain-bytes");
}

#[test]
fn parses_single_byte_and_suffix_ranges() {
    assert_eq!(
        crate::byte_range::parse_range("bytes=0-0", 10),
        Some(Ok((0, 0)))
    );
    assert_eq!(
        crate::byte_range::parse_range("bytes=-3", 10),
        Some(Ok((7, 9)))
    );
    assert_eq!(
        crate::byte_range::parse_range("bytes=4-", 10),
        Some(Ok((4, 9)))
    );
}

#[test]
fn ignores_malformed_and_rejects_unsatisfiable_ranges() {
    assert_eq!(crate::byte_range::parse_range("items=0-1", 10), None);
    assert_eq!(crate::byte_range::parse_range("bytes=bogus", 10), None);
    assert_eq!(
        crate::byte_range::parse_range("bytes=99-100", 10),
        Some(Err(()))
    );
}
