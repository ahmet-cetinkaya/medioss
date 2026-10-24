use axum::{body::Body, http::StatusCode};

use crate::test_support::*;

#[tokio::test]
async fn upload_serves_back_the_normalised_image_with_hardening_headers() {
    let app = app();
    let (status, _) = put_icon(&app, PNG).await;
    assert_eq!(status, StatusCode::OK);

    let (status, headers, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds/1/icon",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, b"RIFF\x04\0\0\0WEBP");
    assert_eq!(
        header_value(&headers, "content-type").as_deref(),
        Some("image/webp")
    );
    assert_eq!(
        header_value(&headers, "x-content-type-options").as_deref(),
        Some("nosniff")
    );
    assert_eq!(
        header_value(&headers, "cache-control").as_deref(),
        Some("no-cache")
    );
}

#[tokio::test]
async fn icon_of_a_sound_without_one_or_an_unknown_sound_is_not_found() {
    let app = app();
    for path in ["/soundscape/sounds/1/icon", "/soundscape/sounds/99/icon"] {
        let (status, _, _) = call(
            &app,
            Some(member_actor()),
            "GET",
            path,
            vec![],
            Body::empty(),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test]
async fn upload_without_an_icon_field_is_a_bad_request() {
    let app = app();
    let body = "--XB\r\nContent-Disposition: form-data; name=\"other\"\r\n\r\nx\r\n--XB--\r\n";
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "PUT",
        "/soundscape/sounds/1/icon",
        vec![("content-type", "multipart/form-data; boundary=XB")],
        Body::from(body),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn clearing_removes_the_icon_and_members_cannot_clear() {
    let app = app();
    put_icon(&app, PNG).await;
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "DELETE",
        "/soundscape/sounds/1/icon",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _, body) = call(
        &app,
        Some(admin_actor()),
        "DELETE",
        "/soundscape/sounds/1/icon",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(String::from_utf8_lossy(&body).contains("\"has_icon\":false"));
}
