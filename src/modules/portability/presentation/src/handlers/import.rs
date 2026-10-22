use crate::extractors::CurrentActor;
use crate::formats;
use crate::router::SharedDependencies;
use crate::support::{
    bad_request, ensure_browser_origin, internal, mood_csv_document, read_import_file,
};
use axum::extract::{Multipart, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use portability_application::commands::import_own_data::{ImportFailure, ImportOwnData};

pub async fn import(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    if let Err(status) = ensure_browser_origin(&headers, &deps.medioss_origin) {
        return (status, "cross-origin requests are not allowed").into_response();
    }
    let bytes = match read_import_file(multipart).await {
        Ok(b) => b,
        Err((status, message)) => return (status, message).into_response(),
    };

    let document = match bytes.iter().find(|b| !b.is_ascii_whitespace()) {
        Some(b'{') => formats::from_json(&bytes),
        _ => mood_csv_document(&bytes),
    }
    .map_err(bad_request);

    let document = match document {
        Ok(d) => d,
        Err(r) => return r,
    };
    match deps
        .import_own_data
        .handle(ImportOwnData {
            user_id: actor.user_id(),
            document,
        })
        .await
    {
        Ok(report) => (StatusCode::OK, Json(report)).into_response(),
        Err(ImportFailure::Validation(message)) => {
            (StatusCode::BAD_REQUEST, message).into_response()
        }
        Err(e) => internal(e.to_string()),
    }
}

/// Mounts this route on the module router.
pub fn register(router: axum::Router<SharedDependencies>) -> axum::Router<SharedDependencies> {
    router.route(
        "/portability/import",
        axum::routing::post(import).layer(axum::extract::DefaultBodyLimit::max(
            crate::support::IMPORT_FILE_CAP_BYTES + 64 * 1024,
        )),
    )
}
