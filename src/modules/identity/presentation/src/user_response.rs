use identity_domain::User;
use kernel::Role;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct UserResponse {
    pub(crate) id: i64,
    pub(crate) email: String,
    pub(crate) role: String,
    pub(crate) must_change_password: bool,
    pub(crate) display_name: Option<String>,
    pub(crate) has_avatar: bool,
}

pub(crate) fn user_response(user: &User) -> UserResponse {
    UserResponse {
        id: user.id().value(),
        email: user.email().as_str().to_owned(),
        role: role_to_str(user.role()).to_owned(),
        must_change_password: user.must_change_password(),
        display_name: user.display_name().map(|name| name.as_str().to_owned()),
        has_avatar: user.avatar().is_some(),
    }
}

pub(crate) fn role_to_str(role: Role) -> &'static str {
    match role {
        Role::Admin => "admin",
        Role::Member => "member",
    }
}

#[cfg(test)]
#[path = "user_response.tests.rs"]
mod tests;
