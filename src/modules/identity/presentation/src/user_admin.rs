use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use identity_application::commands::create_user::CreateUserCommand;
use identity_application::queries::list_users::ListUsersQuery;
use identity_domain::{Email, RawPassword};
use serde::{Deserialize, Serialize};

use crate::actors::{AdminActor, CurrentActor};
use crate::api_error::ApiError;
use crate::dependencies::SharedDependencies;
use crate::origin_guard::ensure_browser_origin;
use crate::user_response::{role_to_str, user_response, UserResponse};

#[derive(Deserialize)]
pub(crate) struct CreateUserRequest {
    email: String,
    password: String,
}

pub(crate) async fn users(
    State(dependencies): State<SharedDependencies>,
    CurrentActor(actor): CurrentActor,
    headers: HeaderMap,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), ApiError> {
    ensure_browser_origin(&headers, &dependencies.medioss_origin)?;
    let email = Email::parse(&request.email)?;
    let temporary_password = RawPassword::parse(&request.password)?;
    let user = dependencies
        .create_user
        .handle(CreateUserCommand {
            actor,
            email,
            temporary_password,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(user_response(&user))))
}

#[derive(Serialize)]
pub(crate) struct UserListItem {
    id: i64,
    email: String,
    role: String,
    must_change_password: bool,
    disabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    display_name: Option<String>,
    has_avatar: bool,
}

pub(crate) async fn users_list(
    _actor: AdminActor,
    State(dependencies): State<SharedDependencies>,
) -> Result<Json<Vec<UserListItem>>, ApiError> {
    let users = dependencies.list_users.handle(ListUsersQuery).await?;
    Ok(Json(
        users
            .into_iter()
            .map(|user| UserListItem {
                id: user.id().value(),
                email: user.email().as_str().to_owned(),
                role: role_to_str(user.role()).to_owned(),
                must_change_password: user.must_change_password(),
                disabled: user.is_disabled(),
                created_at: user.created_at(),
                display_name: user.display_name().map(|name| name.as_str().to_owned()),
                has_avatar: user.avatar().is_some(),
            })
            .collect(),
    ))
}

#[cfg(test)]
#[path = "user_admin.tests.rs"]
mod tests;
