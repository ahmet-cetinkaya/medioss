use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, put},
    Json, Router,
};
use mood_tracker_application::{
    commands,
    queries::{self, MoodRange},
    ApplicationError,
};
use serde::{Deserialize, Serialize};

use crate::{ensure_browser_origin, ApiError, CurrentActor, SharedDependencies};

pub fn mood_tracker_router() -> Router<SharedDependencies> {
    Router::new()
        .route("/mood", get(list).post(create))
        .route("/mood/{id}", put(update).delete(remove))
        .route("/mood/statistics", get(statistics))
        .route("/timeline", get(timeline))
}

#[derive(Deserialize)]
struct MoodRequest {
    score: u8,
    note: Option<String>,
}
#[derive(Deserialize)]
struct RangeQuery {
    range: Option<String>,
    tz_offset_minutes: Option<i16>,
    limit: Option<u16>,
    cursor: Option<String>,
    notes_only: Option<bool>,
}

#[derive(Deserialize)]
struct StatisticsQuery {
    range: Option<String>,
    tz_offset_minutes: Option<i16>,
}
#[derive(Serialize)]
struct EntryResponse {
    id: i64,
    score: u8,
    note: Option<String>,
    recorded_at: String,
    updated_at: String,
}
impl From<mood_tracker_domain::MoodEntry> for EntryResponse {
    fn from(e: mood_tracker_domain::MoodEntry) -> Self {
        Self {
            id: e.id(),
            score: e.score().value(),
            note: e.note().map(ToOwned::to_owned),
            recorded_at: e.recorded_at().to_rfc3339(),
            updated_at: e.updated_at().to_rfc3339(),
        }
    }
}
fn range(range: Option<&str>) -> Result<MoodRange, ApiError> {
    match range.unwrap_or("all") {
        "today" => Ok(MoodRange::Today),
        "last7" => Ok(MoodRange::Last7Days),
        "last30" => Ok(MoodRange::Last30Days),
        "all" => Ok(MoodRange::AllTime),
        _ => Err(ApiError::Application(ApplicationError::InvalidInput(
            "invalid mood range",
        ))),
    }
}
fn offset(tz_offset_minutes: Option<i16>) -> Result<i16, ApiError> {
    let value = tz_offset_minutes.unwrap_or(0);
    // chrono requires a strictly sub-24h offset; ±1440 would always fail
    // FixedOffset::east_opt, so the inclusive bound is 1439.
    if !(-1439..=1439).contains(&value) {
        Err(ApiError::Application(ApplicationError::InvalidInput(
            "invalid timezone offset",
        )))
    } else {
        Ok(value)
    }
}

fn history_cursor(
    value: Option<&str>,
) -> Result<Option<(chrono::DateTime<chrono::Utc>, i64)>, ApiError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let (timestamp, id) =
        value
            .rsplit_once(':')
            .ok_or(ApiError::Application(ApplicationError::InvalidInput(
                "invalid cursor",
            )))?;
    let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp)
        .map_err(|_| ApiError::Application(ApplicationError::InvalidInput("invalid cursor")))?
        .with_timezone(&chrono::Utc);
    let id = id
        .parse()
        .map_err(|_| ApiError::Application(ApplicationError::InvalidInput("invalid cursor")))?;
    Ok(Some((timestamp, id)))
}

fn cursor(
    value: Option<&str>,
) -> Result<Option<(chrono::DateTime<chrono::Utc>, queries::TimelineKind, i64)>, ApiError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let (timestamp, rest) =
        value
            .split_once(':')
            .ok_or(ApiError::Application(ApplicationError::InvalidInput(
                "invalid cursor",
            )))?;
    let (kind, id) =
        rest.rsplit_once(':')
            .ok_or(ApiError::Application(ApplicationError::InvalidInput(
                "invalid cursor",
            )))?;
    let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp)
        .map_err(|_| ApiError::Application(ApplicationError::InvalidInput("invalid cursor")))?
        .with_timezone(&chrono::Utc);
    let kind = match kind {
        "Mood" => queries::TimelineKind::Mood,
        "MeditationNote" => queries::TimelineKind::MeditationNote,
        "MeditationSession" => queries::TimelineKind::MeditationSession,
        _ => {
            return Err(ApiError::Application(ApplicationError::InvalidInput(
                "invalid cursor",
            )))
        }
    };
    let id = id
        .parse()
        .map_err(|_| ApiError::Application(ApplicationError::InvalidInput("invalid cursor")))?;
    Ok(Some((timestamp, kind, id)))
}

async fn create(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    CurrentActor(actor): CurrentActor,
    Json(body): Json<MoodRequest>,
) -> Result<Json<EntryResponse>, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    Ok(Json(
        commands::create_mood_entry(
            deps.entries.as_ref(),
            deps.clock.as_ref(),
            actor.user_id(),
            body.score,
            body.note.as_deref(),
        )
        .await
        .map_err(ApiError::Application)?
        .into(),
    ))
}
async fn update(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    CurrentActor(actor): CurrentActor,
    Path(id): Path<i64>,
    Json(body): Json<MoodRequest>,
) -> Result<Json<EntryResponse>, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    Ok(Json(
        commands::update_mood_entry(
            deps.entries.as_ref(),
            deps.clock.as_ref(),
            actor.user_id(),
            id,
            body.score,
            body.note.as_deref(),
        )
        .await
        .map_err(ApiError::Application)?
        .into(),
    ))
}
async fn remove(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    CurrentActor(actor): CurrentActor,
    Path(id): Path<i64>,
) -> Result<(), ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    commands::delete_mood_entry(deps.entries.as_ref(), actor.user_id(), id)
        .await
        .map_err(ApiError::Application)
}
async fn list(
    State(deps): State<SharedDependencies>,
    CurrentActor(actor): CurrentActor,
    Query(query): Query<RangeQuery>,
) -> Result<Json<HistoryResponse>, ApiError> {
    let page = queries::get_mood_history(
        deps.entries.as_ref(),
        actor.user_id(),
        range(query.range.as_deref())?,
        offset(query.tz_offset_minutes)?,
        deps.clock.now().await,
        query.limit.unwrap_or(50),
        history_cursor(query.cursor.as_deref())?,
    )
    .await
    .map_err(ApiError::Application)?;
    Ok(Json(HistoryResponse {
        items: page.items.into_iter().map(Into::into).collect(),
        next_cursor: page
            .next_cursor
            .map(|(at, id)| format!("{}:{id}", at.to_rfc3339())),
    }))
}
#[derive(Serialize)]
struct HistoryResponse {
    items: Vec<EntryResponse>,
    next_cursor: Option<String>,
}

#[derive(Serialize)]
struct StatisticsResponse {
    entry_count: usize,
    average_score: Option<f64>,
    distribution: [usize; 5],
    distribution_percentages: [f64; 5],
    trend: Vec<Trend>,
}
#[derive(Serialize)]
struct Trend {
    date: String,
    average_score: f64,
}
async fn statistics(
    State(deps): State<SharedDependencies>,
    CurrentActor(actor): CurrentActor,
    Query(query): Query<StatisticsQuery>,
) -> Result<Json<StatisticsResponse>, ApiError> {
    let stats = queries::get_mood_statistics(
        deps.entries.as_ref(),
        actor.user_id(),
        range(query.range.as_deref())?,
        offset(query.tz_offset_minutes)?,
        deps.clock.now().await,
    )
    .await
    .map_err(ApiError::Application)?;
    Ok(Json(StatisticsResponse {
        entry_count: stats.entry_count,
        average_score: stats.average_score,
        distribution: stats.distribution,
        distribution_percentages: stats.distribution.map(|count| {
            if stats.entry_count == 0 {
                0.0
            } else {
                count as f64 * 100.0 / stats.entry_count as f64
            }
        }),
        trend: stats
            .trend
            .into_iter()
            .map(|x| Trend {
                date: x.date.to_string(),
                average_score: x.average_score,
            })
            .collect(),
    }))
}
#[derive(Serialize)]
struct TimelinePageResponse {
    items: Vec<TimelineResponse>,
    next_cursor: Option<String>,
}

#[derive(Serialize)]
struct TimelineResponse {
    id: i64,
    kind: String,
    title: String,
    score: Option<u8>,
    note: Option<String>,
    recorded_at: String,
}
async fn timeline(
    State(deps): State<SharedDependencies>,
    CurrentActor(actor): CurrentActor,
    Query(query): Query<RangeQuery>,
) -> Result<Json<TimelinePageResponse>, ApiError> {
    let limit = query.limit.unwrap_or(50);
    let cursor = cursor(query.cursor.as_deref())?;
    let mut events = queries::get_timeline(
        deps.entries.as_ref(),
        deps.meditation.as_ref(),
        actor.user_id(),
        range(query.range.as_deref())?,
        offset(query.tz_offset_minutes)?,
        deps.clock.now().await,
        limit,
        query.notes_only.unwrap_or(false),
    )
    .await
    .map_err(ApiError::Application)?;
    if let Some((at, kind, id)) = cursor {
        // Keyset continuation: keep only events strictly after the cursor
        // tuple. Survives deletions between pages and keeps equal-timestamp
        // events split deterministically by (kind, id).
        events.retain(|event| (event.recorded_at, event.kind, event.id) > (at, kind, id));
    }
    let has_more = events.len() > usize::from(limit);
    events.truncate(usize::from(limit));
    let next_cursor = has_more.then(|| {
        let last = events.last().expect("continuation implies a final event");
        format!(
            "{}:{:?}:{}",
            last.recorded_at.to_rfc3339(),
            last.kind,
            last.id
        )
    });
    Ok(Json(TimelinePageResponse {
        items: events
            .into_iter()
            .map(|e| TimelineResponse {
                id: e.id,
                kind: format!("{:?}", e.kind),
                title: e.title,
                score: e.score,
                note: e.note,
                recorded_at: e.recorded_at.to_rfc3339(),
            })
            .collect(),
        next_cursor,
    }))
}
