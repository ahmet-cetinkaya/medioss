use std::sync::Arc;

use axum::routing::{get, post};

#[derive(Clone)]
pub struct SharedDependencies {
    pub mood_source: Arc<dyn portability_application::ports::own_mood_records::OwnMoodRecords>,
    pub mood_sink: Arc<dyn portability_application::ports::mood_record_sink::MoodRecordSink>,
    pub practice_source:
        Arc<dyn portability_application::ports::own_practice_records::OwnPracticeRecords>,
    pub practice_sink: Arc<dyn portability_application::ports::practice_sink::PracticeSink>,
    pub meditations: Arc<dyn portability_application::ports::meditation_lookup::MeditationLookup>,
    pub profile:
        Arc<dyn portability_application::ports::display_name_resolver::DisplayNameResolver>,
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
