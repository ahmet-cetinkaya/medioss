use std::sync::Arc;

use axum::Router;
use portability_application::commands::import_own_data::{ImportFailure, ImportOwnDataCommand};
use portability_application::models::{
    export_document::ExportDocument, import_report::ImportReport, mood_record::MoodRecord,
};
use portability_application::ports::command_handler::CommandHandler;
use portability_application::ports::query_handler::QueryHandler;
use portability_application::queries::export_mood_records::ExportMoodRecordsQuery;
use portability_application::queries::export_own_data::ExportOwnDataQuery;

#[derive(Clone)]
pub struct SharedDependencies {
    pub export_own_data:
        Arc<dyn QueryHandler<ExportOwnDataQuery, Output = ExportDocument, Error = String>>,
    pub export_mood_records:
        Arc<dyn QueryHandler<ExportMoodRecordsQuery, Output = Vec<MoodRecord>, Error = String>>,
    pub import_own_data:
        Arc<dyn CommandHandler<ImportOwnDataCommand, Output = ImportReport, Error = ImportFailure>>,
    pub medioss_origin: Arc<String>,
}

pub fn router() -> Router<SharedDependencies> {
    let router = Router::new();
    let router = crate::handlers::export_json::register(router);
    let router = crate::handlers::export_csv::register(router);
    crate::handlers::import::register(router)
}

#[cfg(test)]
#[path = "router.tests.rs"]
mod tests;
