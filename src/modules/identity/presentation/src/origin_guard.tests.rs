use std::sync::{Mutex, MutexGuard, OnceLock};

use axum::http::HeaderMap;

use super::{constant_time_eq, ensure_browser_origin, ensure_setup_secret};

/// MEDIOSS_DEV_ORIGIN is process-global; parallel tests that set it race,
/// so every test touching it holds this lock for its whole body.
fn dev_origin_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

fn headers_with_origin(origin: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("origin", origin.parse().unwrap());
    headers
}

#[test]
fn matching_origin_is_accepted() {
    let headers = headers_with_origin("https://medioss.example");

    assert!(ensure_browser_origin(&headers, "https://medioss.example").is_ok());
}

#[test]
fn origin_match_ignores_case() {
    let headers = headers_with_origin("HTTPS://MEDIoss.EXAMPLE");

    assert!(ensure_browser_origin(&headers, "https://medioss.example").is_ok());
}

#[test]
fn missing_or_mismatched_origin_is_rejected() {
    let empty = HeaderMap::new();
    assert!(ensure_browser_origin(&empty, "https://medioss.example").is_err());

    let other = headers_with_origin("https://evil.example");
    assert!(ensure_browser_origin(&other, "https://medioss.example").is_err());
}

#[test]
fn dev_origin_env_extends_the_allowed_set() {
    let _env = dev_origin_lock();
    let headers = headers_with_origin("http://localhost:4321");
    std::env::set_var("MEDIOSS_DEV_ORIGIN", "http://localhost:4321");

    let accepted = ensure_browser_origin(&headers, "https://medioss.example").is_ok();
    std::env::remove_var("MEDIOSS_DEV_ORIGIN");

    assert!(accepted, "the configured dev origin must be accepted");
}

#[test]
fn empty_dev_origin_env_stays_single_origin() {
    let _env = dev_origin_lock();
    let headers = headers_with_origin("http://localhost:4321");
    std::env::set_var("MEDIOSS_DEV_ORIGIN", "   ");

    let accepted = ensure_browser_origin(&headers, "https://medioss.example").is_ok();
    std::env::remove_var("MEDIOSS_DEV_ORIGIN");

    assert!(!accepted, "an unset/blank dev origin must fail closed");
}

#[test]
fn setup_secret_requires_exact_header() {
    let mut headers = HeaderMap::new();
    headers.insert("x-setup-secret", "s3cret".parse().unwrap());

    assert!(ensure_setup_secret(&headers, "s3cret").is_ok());
    assert!(ensure_setup_secret(&headers, "other").is_err());

    assert!(ensure_setup_secret(&HeaderMap::new(), "s3cret").is_err());
}

#[test]
fn constant_time_eq_behaves_like_equality() {
    assert!(constant_time_eq(b"abc", b"abc"));
    assert!(!constant_time_eq(b"abc", b"abd"));
    assert!(!constant_time_eq(b"abc", b"abcd"));
    assert!(constant_time_eq(b"", b""));
}
