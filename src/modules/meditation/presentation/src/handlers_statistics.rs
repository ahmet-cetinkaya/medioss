use axum::{
    extract::{Query, State},
    Json,
};
use meditation_application::queries::get_user_statistics::{GetUserStatistics, StatisticsRange};
use meditation_application::ApplicationError;
use serde::{Deserialize, Serialize};

use crate::{ApiError, CurrentActor, SharedDependencies};

#[derive(Deserialize)]
pub struct StatisticsQuery {
    /// Preset (`today`, `last30`, `all`); required unless `from`/`to` are sent.
    pub range: Option<String>,
    /// Custom range: inclusive local dates (`YYYY-MM-DD`), sent as a pair.
    pub from: Option<String>,
    pub to: Option<String>,
    pub tz_offset_minutes: i16,
}

#[derive(Serialize)]
pub struct StatisticsResponse {
    /// The preset name, or `custom` for a from/to request.
    pub range: String,
    pub total_minutes: i64,
    pub total_days: i64,
    pub current_streak: i64,
    pub longest_streak: i64,
    pub calendar: Vec<StatisticsDayResponse>,
}

#[derive(Serialize)]
pub struct StatisticsDayResponse {
    pub date: String,
    pub minutes: i64,
}

pub async fn statistics(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Query(query): Query<StatisticsQuery>,
) -> Result<Json<StatisticsResponse>, ApiError> {
    let range = statistics_range(&query)?;
    if !(-1_440..=1_440).contains(&query.tz_offset_minutes) {
        return Err(ApiError::Application(ApplicationError::InvalidInput(
            "invalid timezone offset",
        )));
    }
    let statistics = deps
        .get_user_statistics
        .handle(GetUserStatistics {
            user_id: actor.user_id(),
            range,
            tz_offset_minutes: query.tz_offset_minutes,
        })
        .await
        .map_err(ApiError::Application)?;

    Ok(Json(StatisticsResponse {
        range: query.range.unwrap_or_else(|| "custom".to_owned()),
        total_minutes: statistics.total_minutes,
        total_days: statistics.total_days,
        current_streak: statistics.current_streak,
        longest_streak: statistics.longest_streak,
        calendar: statistics
            .calendar
            .into_iter()
            .map(|day| StatisticsDayResponse {
                date: day.date.format("%Y-%m-%d").to_string(),
                minutes: day.minutes,
            })
            .collect(),
    }))
}

fn statistics_range(query: &StatisticsQuery) -> Result<StatisticsRange, ApiError> {
    match (query.from.as_deref(), query.to.as_deref()) {
        (None, None) => match query.range.as_deref() {
            Some("today") => Ok(StatisticsRange::Today),
            Some("last30") => Ok(StatisticsRange::Last30Days),
            Some("all") => Ok(StatisticsRange::AllTime),
            _ => Err(invalid("invalid statistics range")),
        },
        (Some(from), Some(to)) => {
            if query.range.is_some() {
                return Err(invalid("range cannot be combined with from/to"));
            }
            StatisticsRange::custom(local_date(from)?, local_date(to)?)
                .map_err(ApiError::Application)
        }
        _ => Err(invalid("from and to must be given together")),
    }
}

fn local_date(value: &str) -> Result<chrono::NaiveDate, ApiError> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| invalid("dates must be YYYY-MM-DD"))
}

fn invalid(message: &'static str) -> ApiError {
    ApiError::Application(ApplicationError::InvalidInput(message))
}

#[cfg(test)]
#[path = "handlers_statistics.tests.rs"]
mod tests;
