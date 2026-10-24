use axum::{
    body::Body,
    extract::Multipart,
    http::{Request, StatusCode},
    routing::post,
    Router,
};
use tower::ServiceExt;

use super::read_field;

const FIELD_LIMIT: usize = 8;

async fn read_file(mut multipart: Multipart) -> Result<String, StatusCode> {
    let mut field = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .ok_or(StatusCode::BAD_REQUEST)?;
    let bytes = read_field(&mut field, FIELD_LIMIT).await?;
    Ok(String::from_utf8(bytes).unwrap())
}

async fn upload(payload: &str) -> (StatusCode, String) {
    let body = format!(
        "--XB\r\nContent-Disposition: form-data; name=\"file\"\r\n\r\n{payload}\r\n--XB--\r\n"
    );
    let request = Request::builder()
        .method("POST")
        .uri("/")
        .header("content-type", "multipart/form-data; boundary=XB")
        .body(Body::from(body))
        .unwrap();
    let response = Router::new()
        .route("/", post(read_file))
        .oneshot(request)
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

#[tokio::test]
async fn field_within_the_limit_is_returned_whole() {
    assert_eq!(
        upload("12345678").await,
        (StatusCode::OK, "12345678".to_owned())
    );
}

#[tokio::test]
async fn empty_field_is_returned_empty() {
    assert_eq!(upload("").await, (StatusCode::OK, String::new()));
}

#[tokio::test]
async fn field_over_the_limit_is_payload_too_large() {
    assert_eq!(upload("123456789").await.0, StatusCode::PAYLOAD_TOO_LARGE);
}
