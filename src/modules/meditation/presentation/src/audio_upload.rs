//! Admin audio upload (spec FR-002/003).
use axum::{
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use meditation_application::commands;
use meditation_application::commands::upload_meditation::{
    UploadMeditation, UploadMeditationHandler, UploadMeditationService,
};

use crate::meditation_response::{meditation_response, MeditationResponse};
use crate::response::read_field_capped;
use crate::{ensure_browser_origin, AdminActor, ApiError, SharedDependencies};

/// Longest accepted `program_id` text field: an `i64` fits in 20 characters.
const MAX_PROGRAM_ID_FIELD_BYTES: usize = 32;

/// The multipart fields of an upload request, before validation.
#[derive(Default)]
struct UploadForm {
    name: Option<String>,
    program_id: Option<i64>,
    bytes: Option<Vec<u8>>,
    description: Option<String>,
}

/// An upload form whose required fields are all present.
struct ValidUpload {
    name: String,
    program_id: i64,
    bytes: Vec<u8>,
    description: Option<String>,
}

/// `POST /meditation/audio` — admin-only multipart upload (file + name).
///
/// The 100MB cap is enforced twice: against the multipart field's declared
/// length before its body is accepted (spec FR-002), and against a running
/// byte counter so understated lengths cannot smuggle past (spec FR-003).
pub async fn upload(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Result<(StatusCode, Json<MeditationResponse>), ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    let form = parse_upload_form(multipart).await?;
    let upload = validate_upload_form(form)?;
    let meditation = persist_upload(&deps, upload).await?;
    Ok((StatusCode::CREATED, Json(meditation_response(&meditation))))
}

async fn parse_upload_form(mut multipart: Multipart) -> Result<UploadForm, ApiError> {
    // Text fields are bounded the same way: a streaming field cannot grow
    // past the name limit in memory.
    let max_text = commands::MAX_NAME_LENGTH * 4;
    // Cap above the 2000-char limit (4 bytes/char) so an over-long value
    // reaches the command and gets the 400.
    let max_description = meditation_domain::MAX_DESCRIPTION_LENGTH * 4 + 4;
    let mut form = UploadForm::default();

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
                form.bytes = Some(read_field_capped(&mut field, commands::MAX_UPLOAD_BYTES).await?);
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }
    Ok(form)
}

fn validate_upload_form(form: UploadForm) -> Result<ValidUpload, ApiError> {
    let missing = || ApiError::StatusCode(StatusCode::BAD_REQUEST);
    Ok(ValidUpload {
        name: form.name.ok_or_else(missing)?,
        program_id: form.program_id.ok_or_else(missing)?,
        bytes: form.bytes.ok_or_else(missing)?,
        description: form.description,
    })
}

async fn persist_upload(
    deps: &SharedDependencies,
    upload: ValidUpload,
) -> Result<meditation_domain::Meditation, ApiError> {
    UploadMeditationService::new(
        deps.meditations.clone(),
        deps.memberships.clone(),
        deps.programs.clone(),
        deps.validator.clone(),
        deps.converter.clone(),
        deps.store.clone(),
        deps.clock.clone(),
    )
    .handle(UploadMeditation {
        name: upload.name.clone(),
        program_id: upload.program_id,
        bytes: upload.bytes,
        description: upload.description.clone(),
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
