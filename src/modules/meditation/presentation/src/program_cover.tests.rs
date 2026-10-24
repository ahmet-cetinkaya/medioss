use axum::{body::Body, http::StatusCode};

use crate::router_tests::*;

const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0, 0, 0, 0, 0, 0, 0, 0];

async fn make_program(app: &TestApp, cover: Option<&[u8]>) -> i64 {
    let mut fields: Vec<(&str, &str, &[u8])> = vec![("title", "", b"P")];
    if let Some(bytes) = cover {
        fields.push(("cover", "c.png", bytes));
    }
    let (content_type, body) = multipart_body(&fields);
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

async fn put_cover(
    app: &TestApp,
    actor: Option<kernel::Actor>,
    id: i64,
    fields: &[(&str, &str, &[u8])],
) -> StatusCode {
    let (content_type, body) = multipart_body(fields);
    call(
        app,
        actor,
        "PUT",
        &format!("/meditation/programs/{id}/cover"),
        vec![("content-type", &content_type)],
        Body::from(body),
    )
    .await
    .0
}

#[tokio::test]
async fn cover_is_served_with_its_content_type() {
    let app = app();
    let program = make_program(&app, Some(PNG)).await;
    let (status, headers, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/programs/{program}/cover"),
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(header_value(&headers, "content-type"), Some("image/png"));
    assert!(!body.is_empty());
}

#[tokio::test]
async fn program_without_cover_and_unknown_program_are_404() {
    let app = app();
    let program = make_program(&app, None).await;
    for id in [program, 999] {
        let (status, _, _) = call(
            &app,
            Some(member_actor()),
            "GET",
            &format!("/meditation/programs/{id}/cover"),
            vec![],
            Body::empty(),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND, "program {id}");
    }
}

#[tokio::test]
async fn cover_read_requires_a_session() {
    let app = app();
    let (status, _, _) = call(
        &app,
        None,
        "GET",
        "/meditation/programs/1/cover",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn replace_attaches_a_cover_with_204() {
    let app = app();
    let program = make_program(&app, None).await;
    assert_eq!(
        put_cover(
            &app,
            Some(admin_actor()),
            program,
            &[("cover", "c.png", PNG)]
        )
        .await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(app.store.len(), 1);
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/programs/{program}/cover"),
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn replace_is_admin_only() {
    let app = app();
    let fields: &[(&str, &str, &[u8])] = &[("cover", "c.png", PNG)];
    assert_eq!(
        put_cover(&app, Some(member_actor()), 1, fields).await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        put_cover(&app, None, 1, fields).await,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn replace_without_a_cover_field_is_400() {
    let app = app();
    let program = make_program(&app, None).await;
    assert_eq!(
        put_cover(&app, Some(admin_actor()), program, &[("other", "", b"x")]).await,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn replace_unknown_program_is_404() {
    let app = app();
    assert_eq!(
        put_cover(&app, Some(admin_actor()), 999, &[("cover", "c.png", PNG)]).await,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn replace_oversized_cover_is_413() {
    let app = app();
    let program = make_program(&app, None).await;
    let big = vec![0u8; meditation_application::commands::MAX_COVER_BYTES + 1];
    assert_eq!(
        put_cover(
            &app,
            Some(admin_actor()),
            program,
            &[("cover", "c.png", &big)]
        )
        .await,
        StatusCode::PAYLOAD_TOO_LARGE
    );
}

#[tokio::test]
async fn replace_non_image_is_a_client_error() {
    let app = app();
    let program = make_program(&app, None).await;
    let status = put_cover(
        &app,
        Some(admin_actor()),
        program,
        &[("cover", "c.txt", b"not an image at all")],
    )
    .await;
    assert!(status.is_client_error(), "got {status}");
}

#[tokio::test]
async fn replace_rejects_foreign_origin() {
    let app = app();
    let (content_type, body) = multipart_body(&[("cover", "c.png", PNG)]);
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "PUT",
        "/meditation/programs/1/cover",
        vec![
            ("origin", "http://evil.test"),
            ("content-type", &content_type),
        ],
        Body::from(body),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
