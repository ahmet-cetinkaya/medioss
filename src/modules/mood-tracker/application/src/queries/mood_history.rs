use chrono::{DateTime, Duration, FixedOffset, Utc};
use kernel::UserId;
use mood_tracker_domain::MoodEntry;

use crate::{
    ports::{MoodEntryRepository, Page},
    ApplicationError,
};

pub async fn get_mood_history<R>(
    entries: &R,
    user_id: UserId,
    range: super::MoodRange,
    tz_offset_minutes: i16,
    now: DateTime<Utc>,
    limit: u16,
    cursor: Option<(DateTime<Utc>, i64)>,
) -> Result<Page<MoodEntry>, ApplicationError>
where
    R: MoodEntryRepository + ?Sized,
{
    if !(1..=100).contains(&limit) {
        return Err(ApplicationError::InvalidInput(
            "page size must be between 1 and 100",
        ));
    }
    let offset = FixedOffset::east_opt(i32::from(tz_offset_minutes) * 60)
        .ok_or(ApplicationError::InvalidInput("invalid timezone offset"))?;
    let today = now.with_timezone(&offset).date_naive();
    let start = match range {
        super::MoodRange::Today => Some(today),
        super::MoodRange::Last7Days => Some(today - Duration::days(6)),
        super::MoodRange::Last30Days => Some(today - Duration::days(29)),
        super::MoodRange::AllTime => None,
    };
    let mut items: Vec<_> = entries
        .list_all_for_user(user_id)
        .await
        .map_err(ApplicationError::Repository)?
        .into_iter()
        .filter(|entry| {
            let day = entry.recorded_at().with_timezone(&offset).date_naive();
            day <= today && start.is_none_or(|start| day >= start)
        })
        .collect();
    if let Some((at, id)) = cursor {
        items.retain(|entry| (entry.recorded_at(), entry.id()) < (at, id));
    }
    let has_more = items.len() > usize::from(limit);
    items.truncate(usize::from(limit));
    let next_cursor = has_more.then(|| {
        let last = items.last().expect("continuation implies a final item");
        (last.recorded_at(), last.id())
    });
    Ok(Page { items, next_cursor })
}
