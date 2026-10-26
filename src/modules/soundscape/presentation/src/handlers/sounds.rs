use axum::{
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use soundscape_application::commands::create_sound::CreateSoundCommand;
use soundscape_application::queries::list_sounds::ListSoundsQuery;

use crate::extractors::{admin, AdminActor, CurrentActor};
use crate::router::SharedDependencies;
use crate::support::error_mapping::status_for;
use crate::support::multipart_field::read_field;
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

async fn upload_sound(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<SoundDto>), StatusCode> {
    admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    let mut name = None;
    let mut category_id = None;
    let mut bytes = None;
    let mut icon_key = None;
    let mut icon = None;
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        match field.name() {
            Some("name") => name = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?),
            Some("category_id") => {
                category_id = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| StatusCode::BAD_REQUEST)?
                        .parse()
                        .map_err(|_| StatusCode::BAD_REQUEST)?,
                )
            }
            Some("file") => bytes = Some(read_field(&mut field, deps.max_upload_bytes).await?),
            Some("icon_key") => {
                icon_key = Some(
                    String::from_utf8(
                        read_field(&mut field, 256)
                            .await
                            .map_err(|_| StatusCode::BAD_REQUEST)?,
                    )
                    .map_err(|_| StatusCode::BAD_REQUEST)?,
                )
            }
            Some("icon") => {
                icon = Some(read_field(&mut field, soundscape_domain::MAX_ICON_BYTES).await?)
            }
            _ => {}
        }
    }
    let sound = deps
        .create_sound
        .handle(CreateSoundCommand {
            name: name.ok_or(StatusCode::BAD_REQUEST)?,
            category_id: category_id.ok_or(StatusCode::BAD_REQUEST)?,
            bytes: bytes.ok_or(StatusCode::BAD_REQUEST)?,
            icon_key,
            icon,
        })
        .await
        .map_err(status_for)?;
    Ok((StatusCode::CREATED, Json(sound.into())))
}

pub(crate) fn register(
    router: Router<SharedDependencies>,
    max_upload_bytes: usize,
) -> Router<SharedDependencies> {
    router.route("/soundscape/sounds", get(list_sounds)).route(
        "/soundscape/sounds",
        post(upload_sound).layer(axum::extract::DefaultBodyLimit::max(max_upload_bytes)),
    )
}

#[cfg(test)]
#[path = "sounds.tests.rs"]
mod tests;
