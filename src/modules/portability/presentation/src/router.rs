use crate::export_format;
use crate::extractors::CurrentActor;
use crate::mood_csv;
use axum::extract::{Multipart, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Json;
use portability_application::models::export_document::ExportDocument;
use portability_application::models::practice_snapshot::PracticeSnapshot;
use std::sync::Arc;

const IMPORT_FILE_CAP_BYTES: usize = 10 * 1024 * 1024;

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
}

fn internal(e: String) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, e).into_response()
}

fn bad_request(e: String) -> Response {
    (StatusCode::BAD_REQUEST, e).into_response()
}

fn attachment(name: &str, content_type: &str, body: Vec<u8>) -> Response {
    let mut response = body.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(content_type).unwrap(),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{name}\"")).unwrap(),
    );
    response
}

pub async fn export_json(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Response {
    let document = portability_application::export::export_document(
        actor.user_id(),
        deps.mood_source.as_ref(),
        deps.practice_source.as_ref(),
        deps.profile.as_ref(),
    )
    .await;
    let body = match document {
        Ok(d) => export_format::to_json(&d),
        Err(e) => Err(e),
    };
    match body {
        Ok(bytes) => attachment("medioss-export.json", "application/json", bytes),
        Err(e) => internal(e),
    }
}

pub async fn export_csv(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Response {
    let entries = portability_application::export::export_mood_records(
        actor.user_id(),
        deps.mood_source.as_ref(),
    )
    .await;
    let body = match entries {
        Ok(e) => mood_csv::to_csv(&e),
        Err(e) => Err(e),
    };
    match body {
        Ok(bytes) => attachment("medioss-mood.csv", "text/csv; charset=utf-8", bytes),
        Err(e) => internal(e),
    }
}

pub async fn import(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    multipart: Multipart,
) -> Response {
    let bytes = match read_import_file(multipart).await {
        Ok(b) => b,
        Err((status, message)) => return (status, message).into_response(),
    };

    let document = match bytes.iter().find(|b| !b.is_ascii_whitespace()) {
        Some(b'{') => export_format::from_json(&bytes),
        _ => mood_csv_document(&bytes),
    }
    .map_err(bad_request);

    let document = match document {
        Ok(d) => d,
        Err(r) => return r,
    };
    match portability_application::import::import_document(
        actor.user_id(),
        document,
        deps.mood_source.as_ref(),
        deps.mood_sink.as_ref(),
        deps.practice_sink.as_ref(),
        deps.meditations.as_ref(),
    )
    .await
    {
        Ok(report) => (StatusCode::OK, Json(report)).into_response(),
        Err(e) => internal(e),
    }
}

async fn read_import_file(mut multipart: Multipart) -> Result<Vec<u8>, (StatusCode, String)> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        if field.name() != Some("file") {
            continue;
        }
        let bytes = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        if bytes.len() > IMPORT_FILE_CAP_BYTES {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                "import file exceeds 10 MB".to_string(),
            ));
        }
        return Ok(bytes.to_vec());
    }
    Err((StatusCode::BAD_REQUEST, "missing import file".to_string()))
}

fn mood_csv_document(bytes: &[u8]) -> Result<ExportDocument, String> {
    Ok(ExportDocument {
        format_version: portability_application::export::FORMAT_VERSION,
        exported_at: chrono::Utc::now(),
        display_name: None,
        mood_entries: mood_csv::mood_records_from_csv(bytes)?,
        practice: PracticeSnapshot::default(),
    })
}

pub fn router() -> axum::Router<SharedDependencies> {
    axum::Router::new()
        .route("/portability/export/json", get(export_json))
        .route("/portability/export/csv", get(export_csv))
        .route("/portability/import", post(import))
}
