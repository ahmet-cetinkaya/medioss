use std::collections::BTreeMap;

use chrono::{DateTime, Duration, FixedOffset, NaiveDate, Utc};
use kernel::UserId;

use crate::{ports::MoodEntryRepository, ApplicationError};

/// Longest custom range a caller may ask for, in inclusive days.
pub const MAX_CUSTOM_RANGE_DAYS: i64 = 366;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoodRange {
    Today,
    Last7Days,
    Last30Days,
    AllTime,
    /// Inclusive local dates; build it with [`MoodRange::custom`].
    Custom {
        from: NaiveDate,
        to: NaiveDate,
    },
}

impl MoodRange {
    /// A custom range of inclusive local dates: `from` must not be after
    /// `to`, and the span may not exceed [`MAX_CUSTOM_RANGE_DAYS`].
    pub fn custom(from: NaiveDate, to: NaiveDate) -> Result<Self, ApplicationError> {
        if from > to {
            return Err(ApplicationError::InvalidInput(
                "range start is after its end",
            ));
        }
        if (to - from).num_days() >= MAX_CUSTOM_RANGE_DAYS {
            return Err(ApplicationError::InvalidInput("range is too long"));
        }
        Ok(Self::Custom { from, to })
    }

    /// First local day the range includes; `None` means unbounded.
    pub(crate) fn start(self, today: NaiveDate) -> Option<NaiveDate> {
        match self {
            Self::Today => Some(today),
            Self::Last7Days => Some(today - Duration::days(6)),
            Self::Last30Days => Some(today - Duration::days(29)),
            Self::AllTime => None,
            Self::Custom { from, .. } => Some(from),
        }
    }

    /// Last local day the range includes: today, or the custom end date.
    pub(crate) fn end(self, today: NaiveDate) -> NaiveDate {
        match self {
            Self::Custom { to, .. } => to.min(today),
            _ => today,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoodTrendPoint {
    pub date: NaiveDate,
    pub average_score: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoodStatistics {
    pub entry_count: usize,
    pub average_score: Option<f64>,
    pub distribution: [usize; 5],
    pub trend: Vec<MoodTrendPoint>,
}

pub async fn get_mood_statistics<R>(
    entries: &R,
    user_id: UserId,
    range: MoodRange,
    tz_offset_minutes: i16,
    now: DateTime<Utc>,
) -> Result<MoodStatistics, ApplicationError>
where
    R: MoodEntryRepository + ?Sized,
{
    let offset = FixedOffset::east_opt(i32::from(tz_offset_minutes) * 60)
        .ok_or(ApplicationError::InvalidInput("invalid timezone offset"))?;
    let today = now.with_timezone(&offset).date_naive();
    let start = range.start(today);
    let end = range.end(today);
    let mut distribution = [0; 5];
    let mut by_day: BTreeMap<NaiveDate, (usize, usize)> = BTreeMap::new();
    let mut total = 0_usize;
    for entry in entries
        .list_all_for_user(user_id)
        .await
        .map_err(ApplicationError::Repository)?
    {
        let day = entry.recorded_at().with_timezone(&offset).date_naive();
        if day > end || start.is_some_and(|start| day < start) {
            continue;
        }
        let score = usize::from(entry.score().value());
        total += score;
        distribution[score - 1] += 1;
        let day_values = by_day.entry(day).or_default();
        day_values.0 += score;
        day_values.1 += 1;
    }
    let entry_count = distribution.iter().sum();
    Ok(MoodStatistics {
        entry_count,
        average_score: (entry_count > 0).then(|| total as f64 / entry_count as f64),
        distribution,
        trend: by_day
            .into_iter()
            .map(|(date, (sum, count))| MoodTrendPoint {
                date,
                average_score: sum as f64 / count as f64,
            })
            .collect(),
    })
}
