use super::{attachment, bad_request, internal};
use axum::http::{header, StatusCode};

async fn body_of(response: axum::response::Response) -> Vec<u8> {
    axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec()
}

#[tokio::test]
async fn attachment_sets_content_type_disposition_and_body() {
    let response = attachment("export.json", "application/json", b"{}".to_vec());

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
    assert_eq!(
        response.headers()[header::CONTENT_DISPOSITION],
        "attachment; filename=\"export.json\""
    );
    assert_eq!(body_of(response).await, b"{}");
}

#[tokio::test]
async fn attachment_overrides_the_default_octet_stream_content_type() {
    let response = attachment("a.csv", "text/csv; charset=utf-8", Vec::new());

    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/csv; charset=utf-8"
    );
    assert!(body_of(response).await.is_empty());
}

#[tokio::test]
async fn bad_request_returns_400_with_the_message() {
    let response = bad_request("broken row".to_string());

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_of(response).await, b"broken row");
}

#[tokio::test]
async fn internal_returns_500_without_leaking_the_cause() {
    let response = internal("disk path /secret exploded".to_string());

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body_of(response).await, b"internal error");
}
