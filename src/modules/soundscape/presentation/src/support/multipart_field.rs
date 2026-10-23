use axum::http::StatusCode;

/// Reads one multipart field in bounded chunks instead of field.bytes():
/// DefaultBodyLimit aborts the stream when the declared limit is hit, which
/// lets us answer 413 (too large) instead of a generic 400 (bad multipart).
pub(crate) async fn read_field(
    field: &mut axum::extract::multipart::Field<'_>,
    max: usize,
) -> Result<Vec<u8>, StatusCode> {
    let mut data = Vec::new();
    loop {
        match field.chunk().await {
            Ok(Some(chunk)) => {
                data.extend_from_slice(&chunk);
                if data.len() > max {
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
            Ok(None) => return Ok(data),
            Err(_) => {
                // The stream broke at the body limit on an already large
                // payload means size, not shape; a small payload failing
                // here really is malformed.
                return Err(if data.len() >= 2 * 1024 * 1024 {
                    StatusCode::PAYLOAD_TOO_LARGE
                } else {
                    StatusCode::BAD_REQUEST
                });
            }
        }
    }
}
