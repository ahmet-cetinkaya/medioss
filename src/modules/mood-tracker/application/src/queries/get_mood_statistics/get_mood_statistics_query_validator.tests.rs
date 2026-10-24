use kernel::UserId;

use super::GetMoodStatisticsQueryValidator;
use crate::models::MoodRange;
use crate::queries::get_mood_statistics::GetMoodStatisticsQuery;
use crate::ApplicationError;

fn query(tz_offset_minutes: i16) -> GetMoodStatisticsQuery {
    GetMoodStatisticsQuery {
        user_id: UserId::new(1),
        range: MoodRange::AllTime,
        tz_offset_minutes,
    }
}

#[test]
fn accepts_real_utc_offsets() {
    let validator = GetMoodStatisticsQueryValidator;

    for tz_offset_minutes in [0, 180, -300] {
        assert_eq!(validator.validate(&query(tz_offset_minutes)), Ok(()));
    }
}

#[test]
fn rejects_a_timezone_offset_beyond_a_day() {
    assert_eq!(
        GetMoodStatisticsQueryValidator.validate(&query(24 * 60)),
        Err(ApplicationError::InvalidInput("invalid timezone offset"))
    );
}
