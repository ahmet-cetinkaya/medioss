use super::GetUserStatisticsQueryValidator;
use crate::queries::get_user_statistics::{GetUserStatisticsQuery, StatisticsRange};
use crate::test_support::user_a;
use crate::ApplicationError;

fn query(tz_offset_minutes: i16) -> GetUserStatisticsQuery {
    GetUserStatisticsQuery {
        user_id: user_a(),
        range: StatisticsRange::AllTime,
        tz_offset_minutes,
    }
}

#[test]
fn accepts_a_real_timezone_offset() {
    assert!(GetUserStatisticsQueryValidator
        .validate(&query(180))
        .is_ok());
}

#[test]
fn rejects_an_offset_of_a_day_or_more() {
    let error = GetUserStatisticsQueryValidator
        .validate(&query(24 * 60))
        .expect_err("offset out of range");

    assert!(matches!(
        error,
        ApplicationError::InvalidInput("invalid timezone offset")
    ));
}

#[test]
fn timezone_offset_converts_minutes_to_seconds() {
    let offset = GetUserStatisticsQueryValidator
        .timezone_offset(-90)
        .expect("offset");

    assert_eq!(offset.local_minus_utc(), -5400);
}
