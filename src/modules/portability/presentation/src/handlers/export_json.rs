use crate::extractors::CurrentActor;
use crate::formats;
use crate::router::SharedDependencies;
use crate::support::{attachment, internal};
use axum::extract::State;
use axum::response::Response;
use portability_application::queries::export_own_data::ExportOwnData;

pub async fn export_json(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Response {
    let document = deps
        .export_own_data
        .handle(ExportOwnData {
            user_id: actor.user_id(),
        })
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

/// Mounts this route on the module router.
pub fn register(router: axum::Router<SharedDependencies>) -> axum::Router<SharedDependencies> {
    router.route("/portability/export/json", axum::routing::get(export_json))
}

#[cfg(test)]
#[path = "export_json.tests.rs"]
mod tests;
