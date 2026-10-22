use crate::extractors::CurrentActor;
use crate::formats;
use crate::router::SharedDependencies;
use crate::support::{attachment, internal};
use axum::extract::State;
use axum::response::Response;
use portability_application::queries::export_own_data::ExportOwnDataHandler;

pub async fn export_json(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Response {
    let document = portability_application::queries::export_own_data::ExportOwnDataService::new(
        deps.mood_source.clone(),
        deps.practice_source.clone(),
        deps.profile.clone(),
    )
    .handle(
        portability_application::queries::export_own_data::ExportOwnData {
            user_id: actor.user_id(),
        },
    )
    .await;
    let body = match document {
        Ok(d) => formats::to_json(&d),
        Err(e) => Err(e),
    };
    match body {
        Ok(bytes) => attachment("medioss-export.json", "application/json", bytes),
        Err(e) => internal(e),
    }
}
