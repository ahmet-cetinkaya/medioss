//! Meditation JSON shape shared by every meditation-returning surface.
use serde::Serialize;

#[derive(Serialize)]
pub struct MeditationResponse {
    pub id: i64,
    pub name: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub duration_ms: i64,
    pub created_at: String,
    pub description: Option<String>,
    /// Every meditation surface must show progress: `0` and `false` when the
    /// caller never touched this meditation.
    pub completed: bool,
    pub progress_percent: i64,
    /// The meditation's home program: set on the daily-suggestion and library
    /// responses (cover borrowing, \"no program\" detection); absent elsewhere.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program_id: Option<i64>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub program_has_cover: bool,
    /// The home program as the Player needs it (cover borrowing, link label);
    /// set on the detail response, absent elsewhere.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<ProgramSummary>,
}

/// The part of a program the Player screen renders: its identity, the title
/// for the program link's label, and whether a cover can be borrowed.
#[derive(Serialize)]
pub struct ProgramSummary {
    pub id: i64,
    pub title: String,
    pub has_cover: bool,
}

pub(crate) fn meditation_response(
    meditation: &meditation_domain::Meditation,
) -> MeditationResponse {
    MeditationResponse {
        id: meditation.id(),
        name: meditation.name().to_owned(),
        content_type: meditation.content_type().to_owned(),
        size_bytes: meditation.size_bytes(),
        duration_ms: meditation.duration_ms(),
        created_at: meditation.created_at().to_rfc3339(),
        description: meditation.description().map(str::to_owned),
        completed: false,
        progress_percent: 0,
        program_id: None,
        program_has_cover: false,
        program: None,
    }
}

/// Fills a meditation response's `completed`/`progress_percent` from the
/// caller's own playback state; every meditation surface must show progress,
/// so this is not optional decoration.
pub(crate) async fn with_playback(
    get_playback_state: &dyn meditation_application::queries::get_playback_state::GetPlaybackStateHandler,
    user_id: kernel::UserId,
    meditation: &meditation_domain::Meditation,
) -> MeditationResponse {
    use meditation_application::queries::get_playback_state::GetPlaybackState;
    let mut response = meditation_response(meditation);
    // The query zeroes never-played state itself (spec FR-016), so one call
    // always yields the fields to render.
    if let Ok(state) = get_playback_state
        .handle(GetPlaybackState {
            user_id,
            meditation_id: meditation.id(),
        })
        .await
    {
        response.completed = state.completed;
        response.progress_percent = if state.completed {
            100
        } else if meditation.duration_ms() > 0 {
            (((state.position_ms as i128) * 100) / meditation.duration_ms() as i128).clamp(0, 100)
                as i64
        } else {
            0
        };
    }
    response
}

#[cfg(test)]
#[path = "meditation_response.tests.rs"]
mod tests;
