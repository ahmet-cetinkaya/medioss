use std::sync::Arc;

use axum::Router;
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

pub fn router() -> Router<SharedDependencies> {
    let router = Router::new();
    let router = crate::handlers::export_json::register(router);
    let router = crate::handlers::export_csv::register(router);
    crate::handlers::import::register(router)
}
