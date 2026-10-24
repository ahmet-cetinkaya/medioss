use chrono::NaiveDate;

use super::{MoodRange, MAX_CUSTOM_RANGE_DAYS};
use crate::ApplicationError;

fn date((year, month, day): (i32, u32, u32)) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

#[test]
fn custom_rejects_a_range_that_starts_after_it_ends() {
    let from = date((2026, 5, 10));
    let to = date((2026, 5, 9));
    assert_eq!(
        MoodRange::custom(from, to),
        Err(ApplicationError::InvalidInput(
            "range start is after its end"
        ))
    );
}

#[test]
fn custom_rejects_a_span_of_max_custom_range_days_between_the_bounds() {
    // (to - from).num_days() is exclusive of one bound: 366 there means 367
    // inclusive days, which MAX_CUSTOM_RANGE_DAYS forbids.
    assert_eq!(MAX_CUSTOM_RANGE_DAYS, 366);
    let from = date((2026, 1, 1));
    let to = date((2027, 1, 2));
    assert_eq!(
        MoodRange::custom(from, to),
        Err(ApplicationError::InvalidInput("range is too long"))
    );
}

#[test]
fn custom_accepts_the_longest_span_of_max_custom_range_days_inclusive() {
    let from = date((2026, 1, 1));
    let to = date((2027, 1, 1));
    assert_eq!(
        MoodRange::custom(from, to),
        Ok(MoodRange::Custom { from, to })
    );
}

#[test]
fn today_covers_only_today() {
    let today = date((2026, 10, 24));
    let range = MoodRange::Today;
    assert_eq!(range.start(today), Some(today));
    assert_eq!(range.end(today), today);
}

#[test]
fn last7days_covers_seven_days_ending_today() {
    let today = date((2026, 10, 24));
    let range = MoodRange::Last7Days;
    assert_eq!(range.start(today), Some(date((2026, 10, 18))));
    assert_eq!(range.end(today), today);
}

#[test]
fn last30days_covers_thirty_days_ending_today() {
    let today = date((2026, 10, 24));
    let range = MoodRange::Last30Days;
    assert_eq!(range.start(today), Some(date((2026, 9, 25))));
    assert_eq!(range.end(today), today);
}

#[test]
fn alltime_is_unbounded_at_the_start() {
    let today = date((2026, 10, 24));
    let range = MoodRange::AllTime;
    assert_eq!(range.start(today), None);
    assert_eq!(range.end(today), today);
}

#[test]
fn custom_is_clamped_to_today_at_the_end() {
    let today = date((2026, 10, 24));
    let range = MoodRange::custom(date((2026, 1, 1)), date((2026, 12, 31))).unwrap();
    assert_eq!(range.start(today), Some(date((2026, 1, 1))));
    assert_eq!(range.end(today), today);
}
