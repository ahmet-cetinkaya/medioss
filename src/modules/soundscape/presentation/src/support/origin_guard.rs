use axum::http::{HeaderMap, StatusCode};

pub(crate) fn origin(headers: HeaderMap, expected: &str) -> Result<(), StatusCode> {
    let Some(value) = headers.get("origin").and_then(|v| v.to_str().ok()) else {
        return Err(StatusCode::FORBIDDEN);
    };
    if value.eq_ignore_ascii_case(expected) {
        return Ok(());
    }
    // Dev-only extra origin (the Astro dev server on :4321 proxying to this
    // API). Fail-closed: unset/empty env keeps single-origin behavior.
    if let Ok(extra) = std::env::var("MEDIOSS_DEV_ORIGIN") {
        let extra = extra.trim();
        if !extra.is_empty() && value.eq_ignore_ascii_case(extra) {
            return Ok(());
        }
    }
    Err(StatusCode::FORBIDDEN)
}
