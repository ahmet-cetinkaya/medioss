use super::{ensure_browser_origin, origin_matches};
use axum::http::{HeaderMap, StatusCode};

fn headers_with(origin: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("origin", origin.parse().unwrap());
    headers
}

#[test]
fn same_origin_passes() {
    assert_eq!(
        ensure_browser_origin(
            &headers_with("https://medioss.example"),
            "https://medioss.example"
        ),
        Ok(())
    );
}

#[test]
fn missing_origin_rejected() {
    assert_eq!(
        ensure_browser_origin(&HeaderMap::new(), "https://medioss.example"),
        Err(StatusCode::FORBIDDEN)
    );
}

#[test]
fn cross_origin_rejected() {
    assert_eq!(
        ensure_browser_origin(
            &headers_with("https://evil.example"),
            "https://medioss.example"
        ),
        Err(StatusCode::FORBIDDEN)
    );
}

#[test]
fn origin_match_is_case_insensitive() {
    assert!(origin_matches(
        "HTTPS://MEDIOSs.EXAMPLE",
        "https://medioss.example"
    ));
}
