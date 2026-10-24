use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, put},
    Json, Router,
};
use mood_tracker_application::commands::create_mood_entry::CreateMoodEntry;
use mood_tracker_application::commands::delete_mood_entry::DeleteMoodEntry;
use mood_tracker_application::commands::update_mood_entry::UpdateMoodEntry;
use mood_tracker_application::models::MoodRange;
use mood_tracker_application::queries::get_mood_history::MoodHistory;
use mood_tracker_application::queries::get_mood_statistics::GetMoodStatistics;
use mood_tracker_application::queries::get_timeline::{GetTimeline, TimelineKind};
use mood_tracker_application::ApplicationError;
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
    from: Option<String>,
    to: Option<String>,
    tz_offset_minutes: Option<i16>,
    limit: Option<u16>,
    cursor: Option<String>,
    notes_only: Option<bool>,
}

#[derive(Deserialize)]
struct StatisticsQuery {
    range: Option<String>,
    from: Option<String>,
    to: Option<String>,
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
/// Preset (`range=today|last7|last30|all`, default `all`) or a custom range
/// of inclusive local dates (`from=YYYY-MM-DD&to=YYYY-MM-DD`). The two forms
/// are exclusive: a request that sends `from`/`to` together with `range` is
/// rejected, and `from` and `to` must come as a pair.
fn range(range: Option<&str>, from: Option<&str>, to: Option<&str>) -> Result<MoodRange, ApiError> {
    match (from, to) {
        (None, None) => preset_range(range.unwrap_or("all")),
        (Some(from), Some(to)) => {
            if range.is_some() {
                return Err(invalid("range cannot be combined with from/to"));
            }
            MoodRange::custom(local_date(from)?, local_date(to)?).map_err(ApiError::Application)
        }
        _ => Err(invalid("from and to must be given together")),
    }
}
fn preset_range(name: &str) -> Result<MoodRange, ApiError> {
    match name {
        "today" => Ok(MoodRange::Today),
        "last7" => Ok(MoodRange::Last7Days),
        "last30" => Ok(MoodRange::Last30Days),
        "all" => Ok(MoodRange::AllTime),
        _ => Err(invalid("invalid mood range")),
    }
}
fn local_date(value: &str) -> Result<chrono::NaiveDate, ApiError> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| invalid("dates must be YYYY-MM-DD"))
}
fn invalid(message: &'static str) -> ApiError {
    ApiError::Application(ApplicationError::InvalidInput(message))
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
) -> Result<Option<(chrono::DateTime<chrono::Utc>, TimelineKind, i64)>, ApiError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let (timestamp_and_kind, id) =
        value
            .rsplit_once(':')
            .ok_or(ApiError::Application(ApplicationError::InvalidInput(
                "invalid cursor",
            )))?;
    let (timestamp, kind) = timestamp_and_kind
        .rsplit_once(':')
        .ok_or(ApiError::Application(ApplicationError::InvalidInput(
            "invalid cursor",
        )))?;
    let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp)
        .map_err(|_| ApiError::Application(ApplicationError::InvalidInput("invalid cursor")))?
        .with_timezone(&chrono::Utc);
    let kind = match kind {
        "Mood" => TimelineKind::Mood,
        "MeditationNote" => TimelineKind::MeditationNote,
        "MeditationSession" => TimelineKind::MeditationSession,
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
        deps.create
            .handle(CreateMoodEntry {
                user_id: actor.user_id(),
                score: body.score,
                note: body.note,
            })
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
        deps.update
            .handle(UpdateMoodEntry {
                user_id: actor.user_id(),
                entry_id: id,
                score: body.score,
                note: body.note,
            })
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
    deps.delete
        .handle(DeleteMoodEntry {
            user_id: actor.user_id(),
            entry_id: id,
        })
        .await
        .map_err(ApiError::Application)
}
async fn list(
    State(deps): State<SharedDependencies>,
    CurrentActor(actor): CurrentActor,
    Query(query): Query<RangeQuery>,
) -> Result<Json<HistoryResponse>, ApiError> {
    let page = deps
        .get_mood_history
        .handle(MoodHistory {
            user_id: actor.user_id(),
            range: range(
                query.range.as_deref(),
                query.from.as_deref(),
                query.to.as_deref(),
            )?,
            tz_offset_minutes: offset(query.tz_offset_minutes)?,
            limit: query.limit.unwrap_or(50),
            cursor: history_cursor(query.cursor.as_deref())?,
        })
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
    let stats = deps
        .get_mood_statistics
        .handle(GetMoodStatistics {
            user_id: actor.user_id(),
            range: range(
                query.range.as_deref(),
                query.from.as_deref(),
                query.to.as_deref(),
            )?,
            tz_offset_minutes: offset(query.tz_offset_minutes)?,
        })
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
    let mut events = deps
        .get_timeline
        .handle(GetTimeline {
            user_id: actor.user_id(),
            range: range(
                query.range.as_deref(),
                query.from.as_deref(),
                query.to.as_deref(),
            )?,
            tz_offset_minutes: offset(query.tz_offset_minutes)?,
            limit,
            notes_only: query.notes_only.unwrap_or(false),
        })
        .await
        .map_err(ApiError::Application)?;
    if let Some((at, kind, id)) = cursor {
        // Keyset continuation over the query's descending order
        // (recorded_at DESC, kind ASC, id DESC): keep only events strictly
        // after the cursor tuple. Survives deletions between pages and keeps
        // equal-timestamp events split deterministically by (kind, id).
        events.retain(|event| {
            event.recorded_at < at
                || (event.recorded_at == at
                    && (event.kind > kind || (event.kind == kind && event.id < id)))
        });
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
