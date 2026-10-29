//! Admin audio upload (spec FR-002/003).
use axum::{
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use meditation_application::commands;
use meditation_application::commands::upload_meditation::UploadMeditationCommand;
use meditation_application::ports::StagedMedia;

use crate::meditation_response::{meditation_response, MeditationResponse};
use crate::response::{read_field_capped, stage_field_capped};
use crate::{ensure_browser_origin, AdminActor, ApiError, SharedDependencies};

/// Longest accepted `program_id` text field: an `i64` fits in 20 characters.
const MAX_PROGRAM_ID_FIELD_BYTES: usize = 32;

/// The multipart fields of an upload request, before validation.
#[derive(Default)]
struct UploadForm {
    name: Option<String>,
    program_id: Option<i64>,
    media: Option<StagedMedia>,
    description: Option<String>,
}

/// An upload form whose required fields are all present.
struct ValidUpload {
    name: String,
    program_id: i64,
    media: StagedMedia,
    description: Option<String>,
}

/// `POST /meditation/audio` — admin-only multipart upload (file + name).
///
/// The configured upload cap is enforced twice: the route's body-limit layer answers
/// 413 from a declared `Content-Length` over the cap before any body byte is read
/// (spec FR-002), and a running byte counter aborts a stream that exceeds it, so
/// understated lengths cannot smuggle past (spec FR-003).
pub async fn upload(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Result<(StatusCode, Json<MeditationResponse>), ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    let mut form = UploadForm::default();
    if let Err(error) = read_upload_form(multipart, &deps, &mut form).await {
        discard_staged(&deps, form.media.as_ref()).await;
        return Err(error);
    }
    let staged = form.media.clone();
    let outcome = match validate_upload_form(form) {
        Ok(upload) => persist_upload(&deps, upload).await,
        Err(error) => Err(error),
    };
    match outcome {
        Ok(meditation) => Ok((StatusCode::CREATED, Json(meditation_response(&meditation)))),
        Err(error) => {
            discard_staged(&deps, staged.as_ref()).await;
            Err(error)
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
) -> Result<(), ApiError> {
    // Text fields are bounded the same way: a streaming field cannot grow
    // past the name limit in memory.
    let max_text = commands::MAX_NAME_LENGTH * 4;
    // Cap above the 2000-char limit (4 bytes/char) so an over-long value
    // reaches the command and gets the 400.
    let max_description = meditation_domain::MAX_DESCRIPTION_LENGTH * 4 + 4;
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::StatusCode(StatusCode::BAD_REQUEST))?
    {
        match field.name() {
            Some("program_id") => {
                let text = read_field_capped(&mut field, MAX_PROGRAM_ID_FIELD_BYTES).await?;
                form.program_id = Some(
                    String::from_utf8_lossy(&text)
                        .parse::<i64>()
                        .map_err(|_| ApiError::StatusCode(StatusCode::BAD_REQUEST))?,
                );
            }
            Some("name") => {
                let text = read_field_capped(&mut field, max_text).await?;
                form.name = Some(String::from_utf8_lossy(&text).into_owned());
            }
            Some("description") => {
                let text = read_field_capped(&mut field, max_description).await?;
                form.description = Some(String::from_utf8_lossy(&text).into_owned());
            }
            Some("file") => {
                // A repeated file field replaces the earlier one.
                discard_staged(deps, form.media.as_ref()).await;
                form.media = Some(
                    stage_field_capped(&mut field, deps.staging.as_ref(), deps.max_upload_bytes)
                        .await?,
                );
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }
    Ok(())
}

fn validate_upload_form(form: UploadForm) -> Result<ValidUpload, ApiError> {
    let missing = || ApiError::StatusCode(StatusCode::BAD_REQUEST);
    Ok(ValidUpload {
        name: form.name.ok_or_else(missing)?,
        program_id: form.program_id.ok_or_else(missing)?,
        media: form.media.ok_or_else(missing)?,
        description: form.description,
    })
}

async fn persist_upload(
    deps: &SharedDependencies,
    upload: ValidUpload,
) -> Result<meditation_domain::Meditation, ApiError> {
    deps.upload_meditation
        .handle(UploadMeditationCommand {
            name: upload.name,
            program_id: upload.program_id,
            media: upload.media,
            description: upload.description,
        })
        .await
        .map_err(ApiError::Application)
}

/// Test-only seeding: bypasses HTTP and the router to place one stored
/// meditation into the fakes (router.tests.rs).
#[cfg(test)]
pub(crate) async fn upload_for_tests(
    state: &meditation_application::test_support::RepoState,
    store: &meditation_application::test_support::FakeBlobStore,
    bytes: Vec<u8>,
) -> meditation_domain::Meditation {
    use meditation_application::ports::BlobStore as _;
    let blob = store.put("holder", "seed", "opus", &bytes).await.unwrap();
    meditation_domain::Meditation::new(
        1,
        "Seed",
        meditation_domain::STORED_CONTENT_TYPE,
        blob.size_bytes,
        &blob.storage_key,
        60_000,
        chrono::Utc::now(),
    )
    .unwrap()
    .tap(|row| state.seed_meditation(row.clone()))
}

#[cfg(test)]
trait Tap: Sized {
    fn tap(self, f: impl FnOnce(&Self)) -> Self {
        f(&self);
        self
    }
}
#[cfg(test)]
impl<T> Tap for T {}

#[cfg(test)]
#[path = "audio_upload.tests.rs"]
mod tests;
