use chrono::{Duration, TimeZone, Utc};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use kernel::UserId;
use tower::ServiceExt;

use meditation_application::{ports::SessionRepository, test_support::FakeSessionRepository};

use super::helpers::*;

#[tokio::test]
async fn statistics_returns_only_the_callers_sessions() {
    let test_app = app();
    let sessions = FakeSessionRepository(test_app.state.clone());
    let started_at = Utc
        .with_ymd_and_hms(2026, 9, 29, 8, 0, 0)
        .single()
        .expect("test instant");
    sessions
        .record_if_new(
            UserId::new(2),
            1,
            started_at,
            started_at + Duration::minutes(25),
        )
        .await
        .expect("member session");
    sessions
        .record_if_new(
            UserId::new(1),
            2,
            started_at,
            started_at + Duration::minutes(60),
        )
        .await
        .expect("admin session");

    let (status, _, body) = call(
        &test_app,
        Some(member_actor()),
        "GET",
        "/meditation/statistics?range=all&tz_offset_minutes=0",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let body = String::from_utf8(body).expect("json");
    assert!(body.contains("\"total_minutes\":25"));
    assert!(!body.contains("60"));
}

#[tokio::test]
async fn statistics_returns_401_and_rejects_invalid_query_values() {
    let test_app = app();
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::get("/meditation/statistics?range=all&tz_offset_minutes=0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let (status, _, _) = call(
        &test_app,
        Some(member_actor()),
        "GET",
        "/meditation/statistics?range=year&tz_offset_minutes=0",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _, _) = call(
        &test_app,
        Some(member_actor()),
        "GET",
        "/meditation/statistics?range=all&tz_offset_minutes=1441",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

async fn statistics_status(test_app: &TestApp, query: &str) -> StatusCode {
    let (status, _, _) = call(
        test_app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/statistics?{query}&tz_offset_minutes=0"),
        vec![],
        Body::empty(),
    )
    .await;
    status
}

#[tokio::test]
async fn statistics_custom_range_counts_only_the_inclusive_dates() {
    let test_app = app();
    let sessions = FakeSessionRepository(test_app.state.clone());
    for (id, day, minutes) in [(1, 9, 5), (2, 10, 20), (3, 20, 30), (4, 21, 40)] {
        let started_at = Utc
            .with_ymd_and_hms(2026, 9, day, 8, 0, 0)
            .single()
            .expect("test instant");
        sessions
            .record_if_new(
                UserId::new(2),
                id,
                started_at,
                started_at + Duration::minutes(minutes),
            )
            .await
            .expect("session");
    }

    let (status, _, body) = call(
        &test_app,
        Some(member_actor()),
        "GET",
        "/meditation/statistics?from=2026-09-10&to=2026-09-20&tz_offset_minutes=0",
        vec![],
        Body::empty(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let body = String::from_utf8(body).expect("json");
    assert!(body.contains("\"total_minutes\":50"), "{body}");
    assert!(body.contains("\"range\":\"custom\""), "{body}");
}

#[tokio::test]
async fn statistics_custom_range_is_validated_with_a_400_for_every_bad_shape() {
    let test_app = app();
    for (name, query) in [
        ("from after to", "from=2026-09-20&to=2026-09-10"),
        ("more than 366 days", "from=2025-01-01&to=2026-09-30"),
        ("only from", "from=2026-09-10"),
        ("only to", "to=2026-09-10"),
        ("not a date", "from=yesterday&to=today"),
        ("impossible date", "from=2026-02-30&to=2026-03-02"),
        (
            "preset mixed with custom",
            "range=all&from=2026-09-10&to=2026-09-20",
        ),
        ("neither preset nor custom", "tz=0"),
    ] {
        assert_eq!(
            statistics_status(&test_app, query).await,
            StatusCode::BAD_REQUEST,
            "{name}"
        );
    }
    assert_eq!(
        statistics_status(&test_app, "from=2025-09-30&to=2026-09-30").await,
        StatusCode::OK,
        "a 366-day span is the longest valid range"
    );
}
