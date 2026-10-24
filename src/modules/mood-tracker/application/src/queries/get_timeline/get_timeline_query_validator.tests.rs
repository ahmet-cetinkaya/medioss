use kernel::UserId;

use super::GetTimelineQueryValidator;
use crate::models::MoodRange;
use crate::queries::get_timeline::GetTimelineQuery;
use crate::ApplicationError;

fn query(limit: u16, tz_offset_minutes: i16) -> GetTimelineQuery {
    GetTimelineQuery {
        user_id: UserId::new(1),
        range: MoodRange::AllTime,
        tz_offset_minutes,
        limit,
        notes_only: false,
    }
}

#[test]
fn accepts_page_sizes_one_to_one_hundred_and_real_offsets() {
    let validator = GetTimelineQueryValidator;

    assert_eq!(validator.validate(&query(1, 0)), Ok(()));
    assert_eq!(validator.validate(&query(100, 180)), Ok(()));
    assert_eq!(validator.validate(&query(50, -300)), Ok(()));
}

#[test]
fn rejects_page_sizes_outside_one_to_one_hundred() {
    let validator = GetTimelineQueryValidator;

    for limit in [0, 101] {
        assert_eq!(
            validator.validate(&query(limit, 0)),
            Err(ApplicationError::InvalidInput(
                "page size must be between 1 and 100"
            )),
            "limit {limit}"
        );
    }
}

#[test]
fn rejects_a_timezone_offset_beyond_a_day() {
    assert_eq!(
        GetTimelineQueryValidator.validate(&query(10, 24 * 60)),
        Err(ApplicationError::InvalidInput("invalid timezone offset"))
    );
}
