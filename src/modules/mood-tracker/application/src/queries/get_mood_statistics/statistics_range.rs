use chrono::{Duration, NaiveDate};

use crate::ApplicationError;

/// Longest custom range a caller may ask for, in inclusive days.
pub const MAX_CUSTOM_RANGE_DAYS: i64 = 366;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatisticsRange {
    Today,
    Last7Days,
    Last30Days,
    AllTime,
    /// Inclusive local dates; build it with [`StatisticsRange::custom`].
    Custom {
        from: NaiveDate,
        to: NaiveDate,
    },
}

impl StatisticsRange {
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
