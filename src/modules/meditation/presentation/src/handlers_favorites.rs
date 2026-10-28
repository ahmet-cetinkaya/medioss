//! Favorite handlers: favorite, unfavorite, list — caller-scoped throughout.
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use meditation_application::commands::favorite_meditation::FavoriteMeditationCommand;
use meditation_application::commands::unfavorite_meditation::UnfavoriteMeditationCommand;
use meditation_application::queries::list_favorite_meditations::ListFavoriteMeditationsQuery;

use crate::meditation_response::{meditation_response, MeditationResponse, ProgramLabel};
use crate::{ensure_browser_origin, ApiError, CurrentActor, SharedDependencies};

/// `GET /meditation/favorites` — the caller's favorite meditations, most
/// recently favorited first. Every row is `favorite: true` and lists the
/// programs it belongs to in `programs`.
pub async fn favorites(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Result<Json<Vec<MeditationResponse>>, ApiError> {
    let listed = deps
        .list_favorite_meditations
        .handle(ListFavoriteMeditationsQuery {
            user_id: actor.user_id(),
        })
        .await
        .map_err(ApiError::Application)?;
    let responses = listed
        .iter()
        .map(|entry| {
            let mut response = meditation_response(&entry.meditation);
            response.favorite = true;
            response.programs = Some(
                entry
                    .programs
                    .iter()
                    .map(|program| ProgramLabel {
                        id: program.id,
                        title: program.title.clone(),
                    })
                    .collect(),
            );
            response
        })
        .collect();
    Ok(Json(responses))
}

/// `PUT /meditation/{id}/favorite` — idempotent; `404` for an unknown meditation.
pub async fn favorite(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    deps.favorite_meditation
        .handle(FavoriteMeditationCommand {
            user_id: actor.user_id(),
            meditation_id: id,
        })
        .await
        .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /meditation/{id}/favorite` — idempotent; `404` for an unknown meditation.
pub async fn unfavorite(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    deps.unfavorite_meditation
        .handle(UnfavoriteMeditationCommand {
            user_id: actor.user_id(),
            meditation_id: id,
        })
        .await
        .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
#[path = "handlers_favorites.tests.rs"]
mod tests;
