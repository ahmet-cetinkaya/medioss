use crate::extractors::CurrentActor;
use crate::formats;
use crate::router::SharedDependencies;
use crate::support::{attachment, internal};
use axum::extract::State;
use axum::response::Response;
use portability_application::queries::export_mood_records::ExportMoodRecords;

pub async fn export_csv(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Response {
    let entries = deps
        .export_mood_records
        .handle(ExportMoodRecords {
            user_id: actor.user_id(),
        })
        .await;
    let body = match entries {
        Ok(e) => formats::to_csv(&e),
        Err(e) => Err(e),
    };
    match body {
        Ok(bytes) => attachment("medioss-mood.csv", "text/csv; charset=utf-8", bytes),
        Err(e) => internal(e),
    }
}
