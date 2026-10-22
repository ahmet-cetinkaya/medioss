use crate::extractors::CurrentActor;
use crate::router::SharedDependencies;
use crate::support::{bad_request, internal};
use crate::support::{ensure_browser_origin, mood_csv_document, read_import_file};
use axum::extract::{Multipart, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use portability_application::commands::import_own_data::{ImportFailure, ImportOwnDataHandler};

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
        Some(b'{') => crate::formats::from_json(&bytes),
        _ => mood_csv_document(&bytes),
    }
    .map_err(bad_request);

    let document = match document {
        Ok(d) => d,
        Err(r) => return r,
    };
    let service = portability_application::commands::import_own_data::ImportOwnDataService::new(
        deps.mood_source.clone(),
        deps.mood_sink.clone(),
        deps.practice_source.clone(),
        deps.practice_sink.clone(),
        deps.meditations.clone(),
    );
    match service
        .handle(
            portability_application::commands::import_own_data::ImportOwnData {
                user_id: actor.user_id(),
                document,
            },
        )
        .await
    {
        Ok(report) => (StatusCode::OK, Json(report)).into_response(),
        Err(ImportFailure::Validation(message)) => {
            (StatusCode::BAD_REQUEST, message).into_response()
        }
        Err(e) => internal(e.to_string()),
    }
}
