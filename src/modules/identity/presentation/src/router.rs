//! HTTP surface for the identity module: routing.
//!
//! This crate never resolves sessions itself (web-api middleware inserts the
//! `Actor` into request extensions — ARCHITECTURE.md L335-340) and never
//! touches a database; all persistence flows through application handler
//! traits.

use axum::{
    routing::{get, post},
    Router,
};

use crate::auth::{login, logout, me, me_password, setup, setup_status};
use crate::dependencies::SharedDependencies;
use crate::profile::{avatar_get, me_avatar_delete, me_avatar_put, me_profile};
use crate::user_admin::{users, users_list};
use crate::users_lifecycle::{me_delete, user_delete, user_status};

pub fn identity_router() -> Router<SharedDependencies> {
    Router::new()
        .route("/identity/setup-status", get(setup_status))
        .route("/identity/setup", post(setup))
        .route("/identity/login", post(login))
        .route("/identity/logout", post(logout))
        .route("/identity/me", get(me))
        .route("/identity/users", post(users))
        .route("/identity/users", get(users_list))
        .route(
            "/identity/users/{id}/status",
            axum::routing::patch(user_status),
        )
        .route("/identity/users/{id}", axum::routing::delete(user_delete))
        .route("/identity/me", axum::routing::delete(me_delete))
        .route("/identity/me/password", post(me_password))
        .route("/identity/me/profile", axum::routing::patch(me_profile))
        .route("/identity/me/avatar", axum::routing::put(me_avatar_put))
        .route(
            "/identity/me/avatar",
            axum::routing::delete(me_avatar_delete),
        )
        .route("/identity/avatar/{id}", get(avatar_get))
}
