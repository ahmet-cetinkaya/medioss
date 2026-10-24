use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{header, Request};
use axum::Router;
use chrono::{TimeZone, Utc};
use kernel::{Actor, Role, UserId};
use portability_application::commands::import_own_data::{
    ImportFailure, ImportOwnData, ImportOwnDataHandler,
};
use portability_application::models::export_document::ExportDocument;
use portability_application::models::import_report::ImportReport;
use portability_application::models::mood_record::MoodRecord;
use portability_application::models::practice_snapshot::PracticeSnapshot;
use portability_application::queries::export_mood_records::{
    ExportMoodRecords, ExportMoodRecordsHandler,
};
use portability_application::queries::export_own_data::{ExportOwnData, ExportOwnDataHandler};

use crate::router::{router, SharedDependencies};

pub const ORIGIN: &str = "http://medioss.test";
pub const USER_ID: i64 = 42;
const BOUNDARY: &str = "portability-test-boundary";

pub fn mood(score: u8, note: Option<&str>, day: u32) -> MoodRecord {
    MoodRecord {
        score,
        note: note.map(str::to_string),
        recorded_at: Utc.with_ymd_and_hms(2025, 1, day, 8, 30, 0).unwrap(),
    }
}

pub fn document(entries: Vec<MoodRecord>) -> ExportDocument {
    ExportDocument {
        format_version: 1,
        exported_at: Utc.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).unwrap(),
        display_name: Some("Ada".to_string()),
        mood_entries: entries,
        practice: PracticeSnapshot::default(),
    }
}

pub struct StubExportOwnData(pub Result<ExportDocument, String>);

#[async_trait::async_trait]
impl ExportOwnDataHandler for StubExportOwnData {
    async fn handle(&self, _command: ExportOwnData) -> Result<ExportDocument, String> {
        self.0.clone()
    }
}

pub struct StubExportMoodRecords(pub Result<Vec<MoodRecord>, String>);

#[async_trait::async_trait]
impl ExportMoodRecordsHandler for StubExportMoodRecords {
    async fn handle(&self, _command: ExportMoodRecords) -> Result<Vec<MoodRecord>, String> {
        self.0.clone()
    }
}

/// Answers once with the configured outcome and records every command.
pub struct RecordingImport {
    outcome: Mutex<Option<Result<ImportReport, ImportFailure>>>,
    pub received: Mutex<Vec<(i64, ExportDocument)>>,
}

impl RecordingImport {
    pub fn new(outcome: Result<ImportReport, ImportFailure>) -> Arc<Self> {
        Arc::new(Self {
            outcome: Mutex::new(Some(outcome)),
            received: Mutex::new(Vec::new()),
        })
    }
}

#[async_trait::async_trait]
impl ImportOwnDataHandler for RecordingImport {
    async fn handle(&self, command: ImportOwnData) -> Result<ImportReport, ImportFailure> {
        self.received
            .lock()
            .unwrap()
            .push((command.user_id.value(), command.document));
        self.outcome
            .lock()
            .unwrap()
            .take()
            .expect("import handler called twice")
    }
}

pub fn dependencies(
    export_own_data: Result<ExportDocument, String>,
    export_mood_records: Result<Vec<MoodRecord>, String>,
    import: Arc<RecordingImport>,
) -> SharedDependencies {
    SharedDependencies {
        export_own_data: Arc::new(StubExportOwnData(export_own_data)),
        export_mood_records: Arc::new(StubExportMoodRecords(export_mood_records)),
        import_own_data: import,
        medioss_origin: Arc::new(ORIGIN.to_string()),
    }
}

pub fn default_dependencies() -> SharedDependencies {
    dependencies(
        Ok(document(Vec::new())),
        Ok(Vec::new()),
        RecordingImport::new(Ok(ImportReport::default())),
    )
}

pub fn actor() -> Actor {
    Actor::new(UserId::new(USER_ID), Role::Member)
}

/// The module router with `Actor` inserted directly, as `resolve_actor` would.
pub fn authenticated_app(deps: SharedDependencies) -> Router {
    router().with_state(deps).layer(axum::Extension(actor()))
}

pub fn anonymous_app(deps: SharedDependencies) -> Router {
    router().with_state(deps)
}

pub fn get(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

pub fn multipart_file(field: &str, content: &[u8], origin: Option<&str>) -> Request<Body> {
    let mut body = format!(
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{field}\"; filename=\"f.bin\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .into_bytes();
    body.extend_from_slice(content);
    body.extend_from_slice(format!("\r\n--{BOUNDARY}--\r\n").as_bytes());
    let mut builder = Request::builder()
        .method("POST")
        .uri("/portability/import")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={BOUNDARY}"),
        );
    if let Some(origin) = origin {
        builder = builder.header("origin", origin);
    }
    builder.body(Body::from(body)).unwrap()
}

pub async fn body_text(response: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}
