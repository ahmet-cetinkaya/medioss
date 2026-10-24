use chrono::{TimeZone, Utc};

use super::{parse_rfc3339, to_rfc3339};

#[test]
fn parses_offset_back_into_utc() {
    let parsed = parse_rfc3339("2026-09-26T15:30:00+03:00").unwrap();

    assert_eq!(
        parsed,
        Utc.with_ymd_and_hms(2026, 9, 26, 12, 30, 0).unwrap()
    );
}

#[test]
fn parse_rejects_non_timestamps() {
    assert!(parse_rfc3339("not a timestamp").is_err());
    assert!(parse_rfc3339("").is_err());
}

#[test]
fn to_rfc3339_round_trips_through_parse() {
    let original = Utc.with_ymd_and_hms(2026, 10, 24, 9, 0, 0).unwrap();

    assert_eq!(parse_rfc3339(&to_rfc3339(&original)).unwrap(), original);
}
