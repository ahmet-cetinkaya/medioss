use crate::test_support::{
    anonymous_app, authenticated_app, body_text, dependencies, document, get, mood, RecordingImport,
};
use axum::http::{header, StatusCode};
use portability_application::models::import_report::ImportReport;
use portability_application::models::mood_record::MoodRecord;
use tower::ServiceExt;

const URI: &str = "/portability/export/csv";

fn deps_with(result: Result<Vec<MoodRecord>, String>) -> crate::SharedDependencies {
    dependencies(
        Ok(document(Vec::new())),
        result,
        RecordingImport::new(Ok(ImportReport::default())),
    )
}

#[tokio::test]
async fn exports_header_and_one_row_per_entry_as_csv_attachment() {
    let entries = vec![mood(5, Some("great"), 2), mood(2, None, 3)];
    let response = authenticated_app(deps_with(Ok(entries)))
        .oneshot(get(URI))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/csv; charset=utf-8"
    );
    assert_eq!(
        response.headers()[header::CONTENT_DISPOSITION],
        "attachment; filename=\"medioss-mood.csv\""
    );
    let body = body_text(response).await;
    let lines: Vec<&str> = body.lines().collect();
    assert_eq!(lines[0], "recorded_at,score,note");
    assert_eq!(lines[1], "2025-01-02T08:30:00+00:00,5,great");
    assert_eq!(lines[2], "2025-01-03T08:30:00+00:00,2,");
    assert_eq!(lines.len(), 3);
}

#[tokio::test]
async fn empty_export_is_header_only() {
    let response = authenticated_app(deps_with(Ok(Vec::new())))
        .oneshot(get(URI))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        body_text(response).await.trim_end(),
        "recorded_at,score,note"
    );
}

#[tokio::test]
async fn formula_like_notes_are_neutralised_with_a_leading_quote() {
    let response = authenticated_app(deps_with(Ok(vec![mood(3, Some("=SUM(A1)"), 4)])))
        .oneshot(get(URI))
        .await
        .unwrap();

    assert!(body_text(response).await.contains(",3,'=SUM(A1)"));
}

#[tokio::test]
async fn export_without_an_actor_is_401() {
    let response = anonymous_app(deps_with(Ok(Vec::new())))
        .oneshot(get(URI))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn handler_failure_is_a_generic_500() {
    let response = authenticated_app(deps_with(Err("db down".to_string())))
        .oneshot(get(URI))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body_text(response).await, "internal error");
}
