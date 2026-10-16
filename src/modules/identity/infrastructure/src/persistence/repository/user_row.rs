use identity_application::ports::RepositoryError;
use identity_domain::{AvatarRef, DisplayName, Email, PasswordHash, Role, User};
use kernel::UserId;
use sea_orm::Set;

use super::timestamps::{parse_rfc3339, to_rfc3339};
use crate::persistence::users;

pub(super) const ROLE_ADMIN: &str = "admin";
const ROLE_MEMBER: &str = "member";

pub(super) fn role_to_str(role: Role) -> &'static str {
    match role {
        Role::Admin => ROLE_ADMIN,
        Role::Member => ROLE_MEMBER,
    }
}

pub(super) fn role_from_str(raw: &str) -> Result<Role, RepositoryError> {
    match raw {
        ROLE_ADMIN => Ok(Role::Admin),
        ROLE_MEMBER => Ok(Role::Member),
        _ => Err(RepositoryError::OperationFailed),
    }
}

pub(super) fn restore_user(model: users::Model) -> Result<User, RepositoryError> {
    let email = Email::parse(&model.email).map_err(|_| RepositoryError::OperationFailed)?;
    let role = role_from_str(&model.role)?;
    let must_change = model.must_change_password;
    let expires = match (&model.password_expires_at, must_change) {
        (Some(raw), true) => Some(parse_rfc3339(raw)?),
        (None, false) => None,
        _ => return Err(RepositoryError::OperationFailed),
    };
    User::restore(
        UserId::new(model.id),
        email,
        PasswordHash::from_stored(model.password_hash),
        role,
        parse_rfc3339(&model.created_at)?,
        u64::try_from(model.auth_version).map_err(|_| RepositoryError::OperationFailed)?,
        must_change,
        expires,
        model.disabled,
        model
            .display_name
            .map(|value| DisplayName::parse(value).map_err(|_| RepositoryError::OperationFailed))
            .transpose()?,
        match (model.avatar_key, model.avatar_ext.as_deref()) {
            (Some(key), Some(extension)) => {
                Some(AvatarRef::new(key, extension).map_err(|_| RepositoryError::OperationFailed)?)
            }
            (None, Some(extension)) => Some(
                AvatarRef::from_extension(extension)
                    .map_err(|_| RepositoryError::OperationFailed)?,
            ),
            _ => None,
        },
    )
    .map_err(|_| RepositoryError::OperationFailed)
}

pub(super) fn insert_model(user: &User, email_raw: &str) -> users::ActiveModel {
    users::ActiveModel {
        email: Set(email_raw.to_owned()),
        password_hash: Set(user.password_hash().as_str().to_owned()),
        role: Set(role_to_str(user.role()).to_owned()),
        must_change_password: Set(user.must_change_password()),
        // New users are constructed with auth_version 0, far inside i64 range.
        auth_version: Set(i64::try_from(user.auth_version()).unwrap_or_default()),
        password_expires_at: Set(user.password_expires_at().map(|value| to_rfc3339(&value))),
        disabled: Set(user.is_disabled()),
        display_name: Set(user.display_name().map(|name| name.as_str().to_owned())),
        avatar_ext: Set(user.avatar().map(|avatar| avatar.extension().to_owned())),
        avatar_key: Set(user.avatar().map(|avatar| avatar.key().to_owned())),
        created_at: Set(to_rfc3339(&user.created_at())),
        ..Default::default()
    }
}

pub(super) fn is_unique_violation(error: &sea_orm::DbErr) -> bool {
    match error {
        sea_orm::DbErr::Exec(runtime) | sea_orm::DbErr::Query(runtime) => {
            runtime.to_string().contains("UNIQUE constraint failed")
        }
        _ => false,
    }
}
