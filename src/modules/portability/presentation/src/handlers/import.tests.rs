use crate::support::IMPORT_FILE_CAP_BYTES;
use crate::test_support::{
    authenticated_app, body_text, dependencies, document, multipart_file, RecordingImport, ORIGIN,
    USER_ID,
};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use portability_application::commands::import_own_data::ImportFailure;
use portability_application::models::import_report::ImportReport;
use std::sync::Arc;
use tower::ServiceExt;

const VALID_JSON: &str = r#"{"format_version":1,"exported_at":"2025-02-01T00:00:00Z","display_name":null,"mood_entries":[{"score":4,"note":"ok","recorded_at":"2025-01-02T08:30:00Z"}],"practice":{"sessions":[],"notes":[],"playback":[]}}"#;
const VALID_CSV: &str = "recorded_at,score,note\n2025-01-02T08:30:00+00:00,4,ok\n";

fn import_app(
    outcome: Result<ImportReport, ImportFailure>,
) -> (axum::Router, Arc<RecordingImport>) {
    let import = RecordingImport::new(outcome);
    let deps = dependencies(Ok(document(Vec::new())), Ok(Vec::new()), import.clone());
    (authenticated_app(deps), import)
}

fn report() -> ImportReport {
    ImportReport {
        mood_created: 1,
        ..ImportReport::default()
    }
}

#[tokio::test]
async fn json_import_forwards_the_document_and_returns_the_report() {
    let (app, import) = import_app(Ok(report()));

    let response = app
        .oneshot(multipart_file("file", VALID_JSON.as_bytes(), Some(ORIGIN)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body_text(response).await).unwrap();
    assert_eq!(json["mood_created"], 1);
    let received = import.received.lock().unwrap();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].0, USER_ID);
    assert_eq!(received[0].1.mood_entries[0].score, 4);
}

#[tokio::test]
async fn csv_import_is_detected_when_the_file_does_not_start_with_a_brace() {
    let (app, import) = import_app(Ok(report()));

    let response = app
        .oneshot(multipart_file("file", VALID_CSV.as_bytes(), Some(ORIGIN)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let received = import.received.lock().unwrap();
    assert_eq!(received[0].1.mood_entries.len(), 1);
    assert_eq!(received[0].1.mood_entries[0].note.as_deref(), Some("ok"));
    assert!(received[0].1.practice.sessions.is_empty());
}

#[tokio::test]
async fn leading_whitespace_before_a_json_brace_is_still_json() {
    let (app, import) = import_app(Ok(report()));
    let padded = format!("  \n{VALID_JSON}");

    let response = app
        .oneshot(multipart_file("file", padded.as_bytes(), Some(ORIGIN)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(import.received.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn missing_or_foreign_origin_is_403_and_never_reaches_the_handler() {
    for origin in [None, Some("http://evil.test")] {
        let (app, import) = import_app(Ok(report()));

        let response = app
            .oneshot(multipart_file("file", VALID_JSON.as_bytes(), origin))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{origin:?}");
        assert!(import.received.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn malformed_json_is_400() {
    let (app, import) = import_app(Ok(report()));

    let response = app
        .oneshot(multipart_file("file", b"{ not json", Some(ORIGIN)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(body_text(response)
        .await
        .starts_with("not a valid export file"));
    assert!(import.received.lock().unwrap().is_empty());
}

#[tokio::test]
async fn unsupported_format_version_is_400() {
    let (app, _) = import_app(Ok(report()));
    let json = VALID_JSON.replace("\"format_version\":1", "\"format_version\":99");

    let response = app
        .oneshot(multipart_file("file", json.as_bytes(), Some(ORIGIN)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(body_text(response)
        .await
        .contains("unknown export format version 99"));
}

#[tokio::test]
async fn invalid_csv_rows_are_400() {
    for (csv, expected) in [
        (
            "recorded_at,score,note\nyesterday,3,x\n",
            "invalid recorded_at",
        ),
        (
            "recorded_at,score,note\n2025-01-02T08:30:00Z,high,x\n",
            "invalid score",
        ),
    ] {
        let (app, import) = import_app(Ok(report()));

        let response = app
            .oneshot(multipart_file("file", csv.as_bytes(), Some(ORIGIN)))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{csv}");
        assert!(body_text(response).await.contains(expected));
        assert!(import.received.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn missing_file_part_is_400() {
    let (app, _) = import_app(Ok(report()));

    let response = app
        .oneshot(multipart_file("other", VALID_JSON.as_bytes(), Some(ORIGIN)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_text(response).await, "missing import file");
}

#[tokio::test]
async fn non_multipart_body_is_rejected_before_the_handler_runs() {
    let (app, import) = import_app(Ok(report()));
    let request = Request::builder()
        .method("POST")
        .uri("/portability/import")
        .header("origin", ORIGIN)
        .body(Body::from(VALID_JSON))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert!(response.status().is_client_error());
    assert!(import.received.lock().unwrap().is_empty());
}

#[tokio::test]
async fn validation_failure_from_the_application_is_400_with_its_message() {
    let (app, _) = import_app(Err(ImportFailure::Validation("score out of range".into())));

    let response = app
        .oneshot(multipart_file("file", VALID_JSON.as_bytes(), Some(ORIGIN)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_text(response).await, "score out of range");
}

#[tokio::test]
async fn internal_failure_is_a_generic_500() {
    let (app, _) = import_app(Err(ImportFailure::Internal("db down".into())));

    let response = app
        .oneshot(multipart_file("file", VALID_JSON.as_bytes(), Some(ORIGIN)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body_text(response).await, "internal error");
}

#[tokio::test]
async fn file_over_the_cap_but_within_the_body_limit_is_413() {
    let (app, import) = import_app(Ok(report()));
    let oversized = vec![b'a'; IMPORT_FILE_CAP_BYTES + 1];

    let response = app
        .oneshot(multipart_file("file", &oversized, Some(ORIGIN)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert!(import.received.lock().unwrap().is_empty());
}

#[tokio::test]
async fn body_beyond_the_route_limit_is_rejected_without_reaching_the_handler() {
    let (app, import) = import_app(Ok(report()));
    let oversized = vec![b'a'; IMPORT_FILE_CAP_BYTES + 128 * 1024];

    let response = app
        .oneshot(multipart_file("file", &oversized, Some(ORIGIN)))
        .await
        .unwrap();

    assert!(response.status().is_client_error());
    assert!(import.received.lock().unwrap().is_empty());
}
