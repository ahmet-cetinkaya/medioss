use std::sync::Arc;

use axum::routing::{get, post};
use portability_application::commands::import_own_data::ImportOwnDataHandler;
use portability_application::queries::export_mood_records::ExportMoodRecordsHandler;
use portability_application::queries::export_own_data::ExportOwnDataHandler;

#[derive(Clone)]
pub struct SharedDependencies {
    pub export_own_data: Arc<dyn ExportOwnDataHandler>,
    pub export_mood_records: Arc<dyn ExportMoodRecordsHandler>,
    pub import_own_data: Arc<dyn ImportOwnDataHandler>,
    pub medioss_origin: Arc<String>,
}

pub fn router() -> axum::Router<SharedDependencies> {
    axum::Router::new()
        .route(
            "/portability/export/json",
            get(crate::handlers::export_json),
        )
        .route("/portability/export/csv", get(crate::handlers::export_csv))
        .route(
            "/portability/import",
            post(crate::handlers::import).layer(axum::extract::DefaultBodyLimit::max(
                crate::support::IMPORT_FILE_CAP_BYTES + 64 * 1024,
            )),
        )
}
