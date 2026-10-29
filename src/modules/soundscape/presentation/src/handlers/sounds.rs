use axum::{
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use soundscape_application::commands::create_sound::CreateSoundCommand;
use soundscape_application::ports::staged_upload::StagedMedia;
use soundscape_application::queries::list_sounds::ListSoundsQuery;
use tower_http::limit::RequestBodyLimitLayer;

use crate::extractors::{admin, AdminActor, CurrentActor};
use crate::router::SharedDependencies;
use crate::support::error_mapping::status_for;
use crate::support::multipart_field::{read_field, stage_field};
use crate::support::origin_guard::origin;

#[derive(Serialize)]
pub(crate) struct SoundDto {
    id: i64,
    name: String,
    category_id: i64,
    content_type: String,
    size_bytes: i64,
    duration_ms: i64,
    created_at: String,
    builtin: bool,
    icon_key: Option<String>,
    has_icon: bool,
}
impl From<soundscape_domain::Sound> for SoundDto {
    fn from(value: soundscape_domain::Sound) -> Self {
        Self {
            id: value.id,
            name: value.name,
            category_id: value.category_id,
            content_type: value.content_type,
            size_bytes: value.size_bytes,
            duration_ms: value.duration_ms,
            created_at: value.created_at.to_rfc3339(),
            builtin: value.builtin,
            icon_key: value.icon_key,
            has_icon: value.icon.is_some(),
        }
    }
}

async fn list_sounds(
    CurrentActor(_actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Result<Json<Vec<SoundDto>>, StatusCode> {
    deps.list_sounds
        .handle(ListSoundsQuery {})
        .await
        .map(|items| Json(items.into_iter().map(Into::into).collect()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// The multipart fields of a sound upload, before validation.
#[derive(Default)]
struct UploadForm {
    name: Option<String>,
    category_id: Option<i64>,
    media: Option<StagedMedia>,
    icon_key: Option<String>,
    icon: Option<Vec<u8>>,
}

async fn upload_sound(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
    multipart: Multipart,
) -> Result<(StatusCode, Json<SoundDto>), StatusCode> {
    admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    let mut form = UploadForm::default();
    let staged = match read_upload_form(multipart, &deps, &mut form).await {
        Ok(()) => form.media.clone(),
        Err(status) => {
            discard_staged(&deps, form.media.as_ref()).await;
            return Err(status);
        }
    };
    match create_sound(&deps, form).await {
        Ok(sound) => Ok((StatusCode::CREATED, Json(sound.into()))),
        Err(status) => {
            discard_staged(&deps, staged.as_ref()).await;
            Err(status)
        }
    }
}

/// Removes a staged upload that will not reach the use case. Idempotent, and
/// a failed delete only leaves scratch space for the next startup to clear.
async fn discard_staged(deps: &SharedDependencies, media: Option<&StagedMedia>) {
    if let Some(media) = media {
        let _ = deps.staging.discard(media).await;
    }
}

/// Reads every multipart field into `form`. The form is filled in place, so a
/// failure part-way still exposes the file already staged for cleanup.
async fn read_upload_form(
    mut multipart: Multipart,
    deps: &SharedDependencies,
    form: &mut UploadForm,
) -> Result<(), StatusCode> {
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        match field.name() {
            Some("name") => {
                form.name = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?)
            }
            Some("category_id") => {
                form.category_id = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| StatusCode::BAD_REQUEST)?
                        .parse()
                        .map_err(|_| StatusCode::BAD_REQUEST)?,
                )
            }
            Some("file") => {
                // A repeated file field replaces the earlier one.
                discard_staged(deps, form.media.as_ref()).await;
                form.media = Some(
                    stage_field(&mut field, deps.staging.as_ref(), deps.max_upload_bytes).await?,
                )
            }
            Some("icon_key") => {
                form.icon_key = Some(
                    String::from_utf8(
                        read_field(&mut field, 256)
                            .await
                            .map_err(|_| StatusCode::BAD_REQUEST)?,
                    )
                    .map_err(|_| StatusCode::BAD_REQUEST)?,
                )
            }
            Some("icon") => {
                form.icon = Some(read_field(&mut field, soundscape_domain::MAX_ICON_BYTES).await?)
            }
            _ => {}
        }
    }
    Ok(())
}

async fn create_sound(
    deps: &SharedDependencies,
    form: UploadForm,
) -> Result<soundscape_domain::Sound, StatusCode> {
    deps.create_sound
        .handle(CreateSoundCommand {
            name: form.name.ok_or(StatusCode::BAD_REQUEST)?,
            category_id: form.category_id.ok_or(StatusCode::BAD_REQUEST)?,
            media: form.media.ok_or(StatusCode::BAD_REQUEST)?,
            icon_key: form.icon_key,
            icon: form.icon,
        })
        .await
        .map_err(status_for)
}

/// Room for the multipart envelope (boundaries, part headers, the name field)
/// around a file that is exactly at the cap.
const MULTIPART_OVERHEAD_BYTES: usize = 1024 * 1024;

/// Body cap for the sound upload route. `RequestBodyLimitLayer` answers 413 from
/// a declared `Content-Length` over the cap before any body byte is read (spec
/// FR-002); `DefaultBodyLimit` raises axum's 2 MiB extractor default and still
/// counts bytes as they stream in (spec FR-003).
fn upload_body_limit(max_bytes: usize) -> (axum::extract::DefaultBodyLimit, RequestBodyLimitLayer) {
    let cap = max_bytes + MULTIPART_OVERHEAD_BYTES;
    (
        axum::extract::DefaultBodyLimit::max(cap),
        RequestBodyLimitLayer::new(cap),
    )
}

pub(crate) fn register(
    router: Router<SharedDependencies>,
    max_upload_bytes: usize,
) -> Router<SharedDependencies> {
    router.route("/soundscape/sounds", get(list_sounds)).route(
        "/soundscape/sounds",
        post(upload_sound).layer(upload_body_limit(max_upload_bytes)),
    )
}

#[cfg(test)]
#[path = "sounds.tests.rs"]
mod tests;
