use axum::http::HeaderMap;
use identity_application::ApplicationError;

use crate::api_error::ApiError;

pub(crate) fn ensure_browser_origin(headers: &HeaderMap, expected: &str) -> Result<(), ApiError> {
    match headers.get("origin").and_then(|value| value.to_str().ok()) {
        Some(origin) if origin.eq_ignore_ascii_case(expected) => Ok(()),
        // Dev-only extra origin (the Astro dev server on :4321 proxying to
        // this API). Fail-closed: unset/empty env keeps single-origin.
        Some(origin)
            if std::env::var("MEDIOSS_DEV_ORIGIN")
                .map(|extra| {
                    let extra = extra.trim();
                    !extra.is_empty() && origin.eq_ignore_ascii_case(extra)
                })
                .unwrap_or(false) =>
        {
            Ok(())
        }
        // A missing or mismatched Origin is rejected: fail closed rather
        // than assume any Origin-less request is a same-origin browser one.
        _ => Err(ApiError(ApplicationError::Forbidden)),
    }
}

pub(crate) fn ensure_setup_secret(headers: &HeaderMap, expected: &str) -> Result<(), ApiError> {
    let provided = headers
        .get("x-setup-secret")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
        Ok(())
    } else {
        Err(ApiError(ApplicationError::Forbidden))
    }
}

pub(crate) fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0u8, |acc, (l, r)| acc | (l ^ r))
        == 0
}
