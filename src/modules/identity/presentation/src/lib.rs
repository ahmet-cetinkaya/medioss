mod actors;
mod api_error;
mod auth;
mod dependencies;
mod origin_guard;
pub mod profile;
pub mod router;
pub mod session_keys;
mod user_admin;
mod user_response;
pub mod users_lifecycle;

pub use actors::{CurrentActor, CurrentPrincipal, Principal};
pub use dependencies::{IdentityDependencies, SharedDependencies};
pub use router::identity_router;
pub use users_lifecycle::{NoUserDeletion, UserDeletionListener};

#[cfg(test)]
#[path = "router_tests/mod.rs"]
mod router_tests;
