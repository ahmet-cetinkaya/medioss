use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

pub(crate) fn internal(e: String) -> Response {
    tracing::error!(error = %e, "portability operation failed");
    (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
}

pub(crate) fn bad_request(e: String) -> Response {
    (StatusCode::BAD_REQUEST, e).into_response()
}

pub(crate) fn attachment(name: &str, content_type: &str, body: Vec<u8>) -> Response {
    let mut response = body.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(content_type).unwrap(),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{name}\"")).unwrap(),
    );
    response
}

#[cfg(test)]
#[path = "responses.tests.rs"]
mod tests;
