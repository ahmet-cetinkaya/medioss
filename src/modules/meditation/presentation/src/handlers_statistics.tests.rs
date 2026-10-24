use axum::http::StatusCode;
use chrono::NaiveDate;
use meditation_application::queries::get_user_statistics::StatisticsRange;

use super::{local_date, statistics_range, StatisticsQuery};
use crate::router_tests::*;
use crate::ApiError;

fn query(range: Option<&str>, from: Option<&str>, to: Option<&str>) -> StatisticsQuery {
    StatisticsQuery {
        range: range.map(str::to_owned),
        from: from.map(str::to_owned),
        to: to.map(str::to_owned),
        tz_offset_minutes: 0,
    }
}

fn is_invalid_input(result: Result<StatisticsRange, ApiError>) -> bool {
    matches!(
        result,
        Err(ApiError::Application(
            meditation_application::ApplicationError::InvalidInput(_)
        ))
    )
}

#[test]
fn presets_map_to_their_ranges() {
    assert!(matches!(
        statistics_range(&query(Some("today"), None, None)),
        Ok(StatisticsRange::Today)
    ));
    assert!(matches!(
        statistics_range(&query(Some("last30"), None, None)),
        Ok(StatisticsRange::Last30Days)
    ));
    assert!(matches!(
        statistics_range(&query(Some("all"), None, None)),
        Ok(StatisticsRange::AllTime)
    ));
}

#[test]
fn missing_or_unknown_preset_is_invalid() {
    assert!(is_invalid_input(statistics_range(&query(None, None, None))));
    assert!(is_invalid_input(statistics_range(&query(
        Some("week"),
        None,
        None
    ))));
}

#[test]
fn custom_range_needs_both_dates() {
    assert!(is_invalid_input(statistics_range(&query(
        None,
        Some("2026-01-01"),
        None
    ))));
    assert!(is_invalid_input(statistics_range(&query(
        None,
        None,
        Some("2026-01-01")
    ))));
}

#[test]
fn custom_range_cannot_be_combined_with_a_preset() {
    assert!(is_invalid_input(statistics_range(&query(
        Some("today"),
        Some("2026-01-01"),
        Some("2026-01-02")
    ))));
}

#[test]
fn custom_range_parses_inclusive_dates() {
    let range = statistics_range(&query(None, Some("2026-01-01"), Some("2026-01-31")));
    let expected_from = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let expected_to = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
    match range {
        Ok(StatisticsRange::Custom { from, to }) => {
            assert_eq!((from, to), (expected_from, expected_to));
        }
        _ => panic!("expected a custom range"),
    }
}

#[test]
fn reversed_custom_range_is_rejected() {
    assert!(statistics_range(&query(None, Some("2026-02-01"), Some("2026-01-01"))).is_err());
}

#[test]
fn local_date_accepts_only_iso_dates() {
    assert_eq!(
        local_date("2026-03-04").ok(),
        NaiveDate::from_ymd_opt(2026, 3, 4)
    );
    assert!(local_date("04/03/2026").is_err());
    assert!(local_date("2026-13-01").is_err());
    assert!(local_date("").is_err());
}

#[tokio::test]
async fn preset_request_returns_the_full_response_shape() {
    let app = app();
    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/statistics?range=last30&tz_offset_minutes=0",
        "",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let stats: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(stats["range"], "last30");
    for key in [
        "total_minutes",
        "total_days",
        "current_streak",
        "longest_streak",
    ] {
        assert_eq!(stats[key], 0, "{key}");
    }
    assert!(stats["calendar"].is_array());
}

#[tokio::test]
async fn custom_request_reports_range_custom() {
    let app = app();
    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/statistics?from=2026-01-01&to=2026-01-03&tz_offset_minutes=0",
        "",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let stats: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(stats["range"], "custom");
    assert!(stats["calendar"].is_array());
}

#[tokio::test]
async fn out_of_bounds_offset_is_a_bad_request() {
    let app = app();
    for offset in [1441, -1441] {
        let (status, _, _) = json_body(
            &app,
            Some(member_actor()),
            "GET",
            &format!("/meditation/statistics?range=today&tz_offset_minutes={offset}"),
            "",
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "offset {offset}");
    }
    // chrono accepts offsets under 24h, so 1440 passes the handler's
    // inclusive bound but the service still rejects it with 400.
    for (offset, expected) in [(1439, StatusCode::OK), (1440, StatusCode::BAD_REQUEST)] {
        let (status, _, _) = json_body(
            &app,
            Some(member_actor()),
            "GET",
            &format!("/meditation/statistics?range=today&tz_offset_minutes={offset}"),
            "",
        )
        .await;
        assert_eq!(status, expected, "offset {offset}");
    }
}

#[tokio::test]
async fn invalid_range_is_a_bad_request_and_anonymous_is_401() {
    let app = app();
    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/statistics?range=week&tz_offset_minutes=0",
        "",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(serde_json::from_slice::<serde_json::Value>(&body).unwrap()["error"].is_string());
    let (status, _, _) = json_body(
        &app,
        None,
        "GET",
        "/meditation/statistics?range=today&tz_offset_minutes=0",
        "",
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
