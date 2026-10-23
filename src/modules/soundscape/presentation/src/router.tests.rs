use super::test_support::*;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use soundscape_application::ports::media::ImageNormalizationError;
use soundscape_domain::{SoundIcon, MAX_ICON_BYTES};
use tower::ServiceExt;

// --- access ---
#[tokio::test]
async fn unauthenticated_requests_are_rejected() {
    let app = app();
    let (status, _, _) = call(
        &app,
        None,
        "GET",
        "/soundscape/sounds",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _, _) = call(
        &app,
        None,
        "GET",
        "/soundscape/mixes",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_only_routes_forbid_members() {
    let app = app();
    let payload = r#"{"name":"Whatever"}"#;
    let (headers, body) = json(payload);
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "POST",
        "/soundscape/categories",
        headers,
        body,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "category creation is admin-only"
    );
}

#[tokio::test]
async fn mutations_require_matching_origin() {
    let app = app();
    let payload = r#"{"name":"Focus","items":[{"sound_id":1,"volume":50,"position":0}]}"#;
    let (mut headers, body) = json(payload);
    headers.retain(|(name, _)| *name != "origin");
    let mut request = Request::builder()
        .method("POST")
        .uri("/soundscape/mixes")
        .header("origin", "https://evil.example")
        .header("content-type", "application/json");
    for (name, value) in headers {
        request = request.header(name, value);
    }
    let mut request = request.body(body).unwrap();
    request.extensions_mut().insert(member_actor());
    let response = app.router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn listing_sounds_requires_any_signed_in_actor() {
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
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
}

// --- downloads ---
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
        crate::support::byte_range::parse_range("bytes=0-0", 10),
        Some(Ok((0, 0)))
    );
    assert_eq!(
        crate::support::byte_range::parse_range("bytes=-3", 10),
        Some(Ok((7, 9)))
    );
    assert_eq!(
        crate::support::byte_range::parse_range("bytes=4-", 10),
        Some(Ok((4, 9)))
    );
}

#[test]
fn ignores_malformed_and_rejects_unsatisfiable_ranges() {
    assert_eq!(
        crate::support::byte_range::parse_range("items=0-1", 10),
        None
    );
    assert_eq!(
        crate::support::byte_range::parse_range("bytes=bogus", 10),
        None
    );
    assert_eq!(
        crate::support::byte_range::parse_range("bytes=99-100", 10),
        Some(Err(()))
    );
}

// --- icons ---
#[tokio::test]
async fn sound_upload_normalizes_its_icon_once_before_writing_and_stores_webp() {
    let app = app();
    let (status, body) = post_sound(&app, PNG).await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "{}",
        String::from_utf8_lossy(&body)
    );
    assert_eq!(*app.state.normalization_count.lock().unwrap(), 1);
    assert_eq!(
        app.state
            .store
            .lock()
            .unwrap()
            .get("soundscape/icon-2.webp"),
        Some(&b"RIFF\x04\0\0\0WEBP".to_vec())
    );
    assert_eq!(
        app.state.sounds.lock().unwrap()[1]
            .icon
            .as_ref()
            .unwrap()
            .content_type,
        "image/webp"
    );
}

#[tokio::test]
async fn sound_upload_icon_normalization_failure_writes_nothing() {
    let app = app();
    *app.state.normalization_error.lock().unwrap() = Some(ImageNormalizationError::InvalidPayload);

    let (status, _) = post_sound(&app, PNG).await;

    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(*app.state.normalization_count.lock().unwrap(), 1);
    assert!(app.state.store.lock().unwrap().is_empty());
    assert_eq!(app.state.sounds.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn icon_normalization_busy_and_worker_failures_are_not_client_errors() {
    // A busy or crashed image worker is a server-side condition: it must
    // surface as 503/500, never as "unsupported media" (415).
    for (error, expected) in [
        (
            ImageNormalizationError::Busy,
            StatusCode::SERVICE_UNAVAILABLE,
        ),
        (
            ImageNormalizationError::WorkerFailed,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    ] {
        let app = app();
        *app.state.normalization_error.lock().unwrap() = Some(error);

        let (status, _) = post_sound(&app, PNG).await;
        assert_eq!(status, expected, "{error:?}");

        let (status, _) = put_icon(&app, PNG).await;
        assert_eq!(status, expected, "{error:?}");

        assert_eq!(*app.state.normalization_count.lock().unwrap(), 2);
        assert!(app.state.store.lock().unwrap().is_empty());
        assert_eq!(app.state.sounds.lock().unwrap().len(), 1);
    }
}

#[tokio::test]
async fn icon_normalization_output_too_large_is_413() {
    let app = app();
    *app.state.normalization_error.lock().unwrap() = Some(ImageNormalizationError::OutputTooLarge);

    let (status, _) = post_sound(&app, PNG).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);

    let (status, _) = put_icon(&app, PNG).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn icon_upload_roundtrip_serves_the_image_and_flags_the_sound() {
    let app = app();
    let (status, body) = put_icon(&app, PNG).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    assert!(String::from_utf8_lossy(&body).contains("\"has_icon\":true"));

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
    let content_type = headers.iter().find(|(n, _)| n == header::CONTENT_TYPE);
    assert_eq!(content_type.unwrap().1, "image/webp");

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
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds/1/icon",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn icon_upload_refuses_wrong_type_svg_and_oversize() {
    let app = app();
    let (status, _) = put_icon(&app, b"GIF89a-not-allowed").await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let (status, _) = put_icon(&app, b"<svg xmlns='http://www.w3.org/2000/svg'/>").await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE, "svg is refused");
    let mut big = PNG.to_vec();
    big.resize(1024 * 1024 + 1, 0);
    let (status, _) = put_icon(&app, &big).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn icon_mutations_are_admin_only() {
    let app = app();
    let (headers, body) = icon_form(PNG);
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "PUT",
        "/soundscape/sounds/1/icon",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn sound_upload_rejects_an_oversized_icon_key() {
    let app = app();
    let icon_key = "a".repeat(257);
    let body = format!(
        "--B\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\nRain\r\n--B\r\nContent-Disposition: form-data; name=\"category_id\"\r\n\r\n1\r\n--B\r\nContent-Disposition: form-data; name=\"icon_key\"\r\n\r\n{icon_key}\r\n--B\r\nContent-Disposition: form-data; name=\"file\"; filename=\"rain.ogg\"\r\n\r\naudio\r\n--B--\r\n"
    );
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "POST",
        "/soundscape/sounds",
        vec![("content-type", "multipart/form-data; boundary=B")],
        Body::from(body),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn admin_edits_icon_key_and_it_is_validated() {
    let app = app();
    let (headers, body) = json(r#"{"name":"Rain","category_id":1,"icon_key":"rain"}"#);
    let (status, _, body) = call(
        &app,
        Some(admin_actor()),
        "PUT",
        "/soundscape/sounds/1",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    assert!(String::from_utf8_lossy(&body).contains("\"icon_key\":\"rain\""));

    let (headers, body) = json(r#"{"name":"Rain","category_id":1,"icon_key":"Bad Key!"}"#);
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "PUT",
        "/soundscape/sounds/1",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn missing_icon_is_404() {
    let app = app();
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds/1/icon",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn missing_icon_blob_is_404_but_store_failures_are_500() {
    let app = app();
    app.state.sounds.lock().unwrap().first_mut().unwrap().icon = Some(SoundIcon {
        storage_key: "soundscape/icon-1.png".into(),
        content_type: "image/png".into(),
    });

    *app.state.open_error.lock().unwrap() = Some("no blob at soundscape/icon-1.png".into());
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds/1/icon",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    *app.state.open_error.lock().unwrap() =
        Some("io on /media/icon-1.png: permission denied".into());
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds/1/icon",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn an_oversized_stored_icon_is_refused_not_buffered() {
    let app = app();
    // Simulate an out-of-band media-store change: the blob behind the icon
    // grew past the upload limit. Serving must refuse, not buffer it all.
    {
        let mut sounds = app.state.sounds.lock().unwrap();
        let sound = sounds.first_mut().unwrap();
        sound.icon = Some(SoundIcon {
            storage_key: "soundscape/icon-1.png".into(),
            content_type: "image/png".into(),
        });
    }
    let mut oversized = PNG.to_vec();
    oversized.resize(MAX_ICON_BYTES + 1, 0);
    app.state
        .store
        .lock()
        .unwrap()
        .insert("soundscape/icon-1.png".to_owned(), oversized);

    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds/1/icon",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

// --- mixes ---
#[tokio::test]
async fn members_manage_their_own_mixes() {
    let app = app();
    let payload = r#"{"name":"Focus","items":[{"sound_id":1,"volume":50,"position":0}]}"#;
    let (headers, body) = json(payload);
    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "POST",
        "/soundscape/mixes",
        headers,
        body,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "{}",
        String::from_utf8_lossy(&body)
    );

    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/mixes",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let text = String::from_utf8_lossy(&body);
    assert!(
        text.contains("\"name\":\"Focus\""),
        "member sees own mix: {text}"
    );
}

#[tokio::test]
async fn mix_of_another_user_is_invisible_and_untouchable() {
    let app = app();
    let payload = r#"{"name":"Mine","items":[{"sound_id":1,"volume":50,"position":0}]}"#;
    let (headers, body) = json(payload);
    let (status, _, created) = call(
        &app,
        Some(another_member_actor()),
        "POST",
        "/soundscape/mixes",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id: i64 = serde_json::from_slice::<serde_json::Value>(&created).unwrap()["id"]
        .as_i64()
        .unwrap();

    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/soundscape/mixes/{id}"),
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "foreign mix by id is 404");
}
