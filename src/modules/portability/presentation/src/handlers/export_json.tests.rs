use crate::test_support::{
    anonymous_app, authenticated_app, body_text, dependencies, document, get, mood, RecordingImport,
};
use axum::http::{header, StatusCode};
use portability_application::models::import_report::ImportReport;
use tower::ServiceExt;

const URI: &str = "/portability/export/json";

fn deps_with(
    result: Result<portability_application::models::export_document::ExportDocument, String>,
) -> crate::SharedDependencies {
    dependencies(
        result,
        Ok(Vec::new()),
        RecordingImport::new(Ok(ImportReport::default())),
    )
}

#[tokio::test]
async fn exports_the_document_as_a_json_attachment() {
    let doc = document(vec![mood(4, Some("calm"), 3)]);
    let response = authenticated_app(deps_with(Ok(doc)))
        .oneshot(get(URI))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
    assert_eq!(
        response.headers()[header::CONTENT_DISPOSITION],
        "attachment; filename=\"medioss-export.json\""
    );
    let json: serde_json::Value = serde_json::from_str(&body_text(response).await).unwrap();
    assert_eq!(json["format_version"], 1);
    assert_eq!(json["display_name"], "Ada");
    assert_eq!(json["mood_entries"][0]["score"], 4);
    assert_eq!(json["mood_entries"][0]["note"], "calm");
}

#[tokio::test]
async fn empty_export_still_has_the_document_skeleton() {
    let response = authenticated_app(deps_with(Ok(document(Vec::new()))))
        .oneshot(get(URI))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body_text(response).await).unwrap();
    assert_eq!(json["mood_entries"], serde_json::json!([]));
    assert_eq!(json["practice"]["sessions"], serde_json::json!([]));
}

#[tokio::test]
async fn export_without_an_actor_is_401() {
    let response = anonymous_app(deps_with(Ok(document(Vec::new()))))
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
