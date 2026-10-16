use std::sync::Arc;

use identity_domain::PasswordHash;

/// Everything a handler needs, injected by web-api's composition root.
pub struct IdentityDependencies<R, H, C, A> {
    pub repository: Arc<R>,
    pub hasher: Arc<H>,
    pub clock: Arc<C>,
    pub admission: Arc<A>,
    pub dummy_password_hash: PasswordHash,
    pub session_secret: Arc<String>,
    pub medioss_origin: Arc<String>,
    pub deletion: Arc<dyn crate::users_lifecycle::UserDeletionListener>,
    pub avatar_store: Arc<dyn identity_application::ports::AvatarStore>,
}

pub type SharedDependencies<R, H, C, A> = Arc<IdentityDependencies<R, H, C, A>>;
