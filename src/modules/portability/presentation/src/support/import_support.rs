use axum::extract::Multipart;
use axum::http::{HeaderMap, StatusCode};
use portability_application::models::export_document::ExportDocument;
use portability_application::models::practice_snapshot::PracticeSnapshot;

pub(crate) const IMPORT_FILE_CAP_BYTES: usize = 10 * 1024 * 1024;

pub(crate) fn origin_matches(value: &str, origin: &str) -> bool {
    if value.eq_ignore_ascii_case(origin) {
        return true;
    }
    // Dev-only extra origin (the Astro dev server on :4321 proxying to this
    // API). Fail-closed: unset/empty env keeps single-origin behavior.
    if let Ok(extra) = std::env::var("MEDIOSS_DEV_ORIGIN") {
        let extra = extra.trim();
        if !extra.is_empty() && value.eq_ignore_ascii_case(extra) {
            return true;
        }
    }
    false
}

pub(crate) fn ensure_browser_origin(headers: &HeaderMap, origin: &str) -> Result<(), StatusCode> {
    headers
        .get("origin")
        .and_then(|value| value.to_str().ok())
        .filter(|value| origin_matches(value, origin))
        .ok_or(StatusCode::FORBIDDEN)
        .map(|_| ())
}

pub(crate) async fn read_import_file(
    mut multipart: Multipart,
) -> Result<Vec<u8>, (StatusCode, String)> {
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

pub(crate) fn mood_csv_document(bytes: &[u8]) -> Result<ExportDocument, String> {
    Ok(ExportDocument {
        format_version: portability_application::FORMAT_VERSION,
        exported_at: chrono::Utc::now(),
        display_name: None,
        mood_entries: crate::formats::mood_records_from_csv(bytes)?,
        practice: PracticeSnapshot::default(),
    })
}

#[cfg(test)]
#[path = "import_support.tests.rs"]
mod tests;
