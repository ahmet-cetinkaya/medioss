use axum::{
    extract::{Query, State},
    Json,
};
use meditation_application::{
    queries::{self, StatisticsRange},
    ApplicationError,
};
use serde::{Deserialize, Serialize};

use crate::{ApiError, CurrentActor, SharedDependencies};

#[derive(Deserialize)]
pub struct StatisticsQuery {
    pub range: String,
    pub tz_offset_minutes: i16,
}

#[derive(Serialize)]
pub struct StatisticsResponse {
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
    let range = match query.range.as_str() {
        "today" => StatisticsRange::Today,
        "last30" => StatisticsRange::Last30Days,
        "all" => StatisticsRange::AllTime,
        _ => {
            return Err(ApiError::Application(ApplicationError::InvalidInput(
                "invalid statistics range",
            )))
        }
    };
    if !(-1_440..=1_440).contains(&query.tz_offset_minutes) {
        return Err(ApiError::Application(ApplicationError::InvalidInput(
            "invalid timezone offset",
        )));
    }
    let now = deps.clock.now().await;
    let statistics = queries::get_user_statistics(
        deps.sessions.as_ref(),
        actor.user_id(),
        range,
        query.tz_offset_minutes,
        now,
    )
    .await
    .map_err(ApiError::Application)?;

    Ok(Json(StatisticsResponse {
        range: query.range,
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
