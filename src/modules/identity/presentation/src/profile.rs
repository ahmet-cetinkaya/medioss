use axum::extract::{Multipart, Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use identity_application::commands::{clear_avatar, read_avatar, replace_avatar, set_display_name};
use identity_application::ports::{LoginThrottleRepository, UserRepository};
use identity_application::ApplicationError;
use identity_domain::DisplayName;
use kernel::UserId;

use crate::actors::CurrentActor;
use crate::api_error::ApiError;
use crate::dependencies::SharedDependencies;
use crate::origin_guard::ensure_browser_origin;

pub const MAX_AVATAR_BYTES: usize = 2 * 1024 * 1024;

#[derive(serde::Deserialize)]
pub struct ProfileRequest {
    pub display_name: String,
}

#[derive(serde::Serialize)]
pub struct ProfileResponse {
    pub id: i64,
    pub email: String,
    pub display_name: Option<String>,
    pub has_avatar: bool,
}

fn profile_response(user: &identity_domain::User) -> ProfileResponse {
    ProfileResponse {
        id: user.id().value(),
        email: user.email().as_str().to_owned(),
        display_name: user.display_name().map(|name| name.as_str().to_owned()),
        has_avatar: user.avatar().is_some(),
    }
}

fn map_multipart_error(error: axum::extract::multipart::MultipartError) -> ApiError {
    let application_error = if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        ApplicationError::PayloadTooLarge
    } else {
        ApplicationError::InvalidInput(identity_domain::IdentityError::InvalidProfile)
    };
    ApiError::from(application_error)
}

pub async fn me_profile<R, H, C, A>(
    State(dependencies): State<SharedDependencies<R, H, C, A>>,
    CurrentActor(actor): CurrentActor,
    headers: HeaderMap,
    Json(request): Json<ProfileRequest>,
) -> Result<Json<ProfileResponse>, ApiError>
where
    R: UserRepository + LoginThrottleRepository + 'static,
    H: identity_application::ports::PasswordHasher + 'static,
    C: identity_application::ports::Clock + 'static,
    A: identity_application::ports::LoginAdmission + 'static,
{
    ensure_browser_origin(&headers, &dependencies.medioss_origin)?;
    let display_name = if request.display_name.trim().is_empty() {
        None
    } else {
        Some(DisplayName::parse(&request.display_name)?)
    };
    let user = set_display_name(dependencies.repository.as_ref(), &actor, display_name).await?;
    Ok(Json(profile_response(&user)))
}

pub async fn me_avatar_put<R, H, C, A>(
    State(dependencies): State<SharedDependencies<R, H, C, A>>,
    CurrentActor(actor): CurrentActor,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<StatusCode, ApiError>
where
    R: UserRepository + LoginThrottleRepository + 'static,
    H: identity_application::ports::PasswordHasher + 'static,
    C: identity_application::ports::Clock + 'static,
    A: identity_application::ports::LoginAdmission + 'static,
{
    ensure_browser_origin(&headers, &dependencies.medioss_origin)?;

    let mut bytes: Option<Vec<u8>> = None;
    while let Some(mut field) = multipart.next_field().await.map_err(map_multipart_error)? {
        if field.name() != Some("file") {
            return Err(ApiError::from(ApplicationError::InvalidInput(
                identity_domain::IdentityError::InvalidProfile,
            )));
        }
        let mut buffer = Vec::new();
        while let Some(chunk) = field.chunk().await.map_err(map_multipart_error)? {
            buffer.extend_from_slice(&chunk);
            if buffer.len() > MAX_AVATAR_BYTES {
                return Err(ApiError::from(ApplicationError::PayloadTooLarge));
            }
        }
        bytes = Some(buffer);
    }

    let bytes = bytes.ok_or_else(|| {
        ApiError::from(ApplicationError::InvalidInput(
            identity_domain::IdentityError::InvalidProfile,
        ))
    })?;
    replace_avatar(
        dependencies.repository.as_ref(),
        dependencies.avatar_store.as_ref(),
        &actor,
        bytes,
    )
    .await?;
    Ok(StatusCode::CREATED)
}

pub async fn me_avatar_delete<R, H, C, A>(
    State(dependencies): State<SharedDependencies<R, H, C, A>>,
    CurrentActor(actor): CurrentActor,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError>
where
    R: UserRepository + LoginThrottleRepository + 'static,
    H: identity_application::ports::PasswordHasher + 'static,
    C: identity_application::ports::Clock + 'static,
    A: identity_application::ports::LoginAdmission + 'static,
{
    ensure_browser_origin(&headers, &dependencies.medioss_origin)?;
    clear_avatar(
        dependencies.repository.as_ref(),
        dependencies.avatar_store.as_ref(),
        &actor,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn avatar_get<R, H, C, A>(
    State(dependencies): State<SharedDependencies<R, H, C, A>>,
    CurrentActor(actor): CurrentActor,
    Path(id): Path<i64>,
) -> Result<Response, ApiError>
where
    R: UserRepository + LoginThrottleRepository + 'static,
    H: identity_application::ports::PasswordHasher + 'static,
    C: identity_application::ports::Clock + 'static,
    A: identity_application::ports::LoginAdmission + 'static,
{
    let (bytes, avatar) = read_avatar(
        dependencies.repository.as_ref(),
        dependencies.avatar_store.as_ref(),
        UserId::new(id),
        &actor,
    )
    .await?;

    let content_type = match avatar.extension() {
        "webp" => "image/webp",
        _ => "image/png",
    };
    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "private, max-age=86400"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        bytes,
    )
        .into_response())
}
