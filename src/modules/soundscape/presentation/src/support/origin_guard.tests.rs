use axum::http::{HeaderMap, HeaderValue, StatusCode};

use super::origin;

const EXPECTED: &str = "https://medioss.example";

fn headers_with_origin(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("origin", HeaderValue::from_str(value).unwrap());
    headers
}

#[test]
fn missing_origin_header_is_forbidden() {
    assert_eq!(
        origin(HeaderMap::new(), EXPECTED),
        Err(StatusCode::FORBIDDEN)
    );
}

#[test]
fn matching_origin_is_accepted_ignoring_case() {
    assert_eq!(origin(headers_with_origin(EXPECTED), EXPECTED), Ok(()));
    assert_eq!(
        origin(headers_with_origin("HTTPS://MEDIOSS.EXAMPLE"), EXPECTED),
        Ok(())
    );
}

#[test]
fn foreign_origin_is_forbidden() {
    assert_eq!(
        origin(headers_with_origin("https://evil.example"), EXPECTED),
        Err(StatusCode::FORBIDDEN)
    );
}

/// Removes the variable on drop so a failing assertion cannot leak it.
struct EnvVarGuard(&'static str);

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        std::env::remove_var(self.0);
    }
}

// The only test that touches MEDIOSS_DEV_ORIGIN, so it cannot race.
#[test]
fn dev_origin_is_accepted_only_while_the_variable_is_set_and_non_empty() {
    let dev = "http://localhost:4321";
    let _guard = EnvVarGuard("MEDIOSS_DEV_ORIGIN");
    std::env::set_var("MEDIOSS_DEV_ORIGIN", dev);
    let with_dev = origin(headers_with_origin(dev), EXPECTED);
    std::env::set_var("MEDIOSS_DEV_ORIGIN", "  ");
    let blank_matches_blank = origin(headers_with_origin(" "), EXPECTED);
    std::env::remove_var("MEDIOSS_DEV_ORIGIN");
    let unset = origin(headers_with_origin(dev), EXPECTED);

    assert_eq!(with_dev, Ok(()));
    assert_eq!(blank_matches_blank, Err(StatusCode::FORBIDDEN));
    assert_eq!(unset, Err(StatusCode::FORBIDDEN));
}
