use chrono::{Duration, NaiveDate};

use super::{StatisticsRange, MAX_CUSTOM_RANGE_DAYS};

fn local_day(month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, month, day).expect("test date")
}

#[test]
fn custom_statistics_range_accepts_valid_spans_and_rejects_bad_ones() {
    let from = local_day(9, 1);
    assert_eq!(
        StatisticsRange::custom(from, from).expect("one day"),
        StatisticsRange::Custom { from, to: from }
    );
    let last_valid = from + Duration::days(MAX_CUSTOM_RANGE_DAYS - 1);
    assert!(StatisticsRange::custom(from, last_valid).is_ok());
    assert!(StatisticsRange::custom(from, last_valid + Duration::days(1)).is_err());
    assert!(StatisticsRange::custom(local_day(9, 2), from).is_err());
}
