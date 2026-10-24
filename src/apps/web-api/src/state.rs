//! Shared handler state.

use std::sync::Arc;

use identity_application::ports::{
    Clock, LoginAdmission, LoginThrottleRepository, PasswordHasher, UserRepository,
};
use identity_presentation::SharedDependencies;
use tower_sessions_sqlx_store::SqliteStore;

use crate::config::Config;

pub struct AppState {
    pub config: Arc<Config>,
    pub session_store: Arc<SqliteStore>,
    pub meditation: meditation_presentation::SharedDependencies,
    pub mood_tracker: mood_tracker_presentation::SharedDependencies,
    pub soundscape: soundscape_presentation::SharedDependencies,
    pub portability: portability_presentation::SharedDependencies,
    pub identity: SharedDependencies,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            session_store: Arc::clone(&self.session_store),
            meditation: Arc::clone(&self.meditation),
            mood_tracker: Arc::clone(&self.mood_tracker),
            soundscape: Arc::clone(&self.soundscape),
            portability: self.portability.clone(),
            identity: self.identity.clone(),
        }
    }
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: Arc<Config>,
        session_store: Arc<SqliteStore>,
        meditation: meditation_presentation::SharedDependencies,
        mood_tracker: mood_tracker_presentation::SharedDependencies,
        soundscape: soundscape_presentation::SharedDependencies,
        portability: portability_presentation::SharedDependencies,
        repository: Arc<identity_infrastructure::SeaOrmUserRepository>,
        hasher: identity_infrastructure::BoundedArgon2PasswordHasher,
        deletion: Arc<dyn identity_presentation::UserDeletionListener>,
        avatar_store: Arc<crate::avatar_store::FileAvatarStore>,
    ) -> Self {
        let identity = identity_dependencies(repository, hasher, &config, deletion, avatar_store);
        Self {
            config,
            meditation,
            mood_tracker,
            soundscape,
            portability,
            session_store,
            identity,
        }
    }
}

/// Builds the identity use-case services from concrete infrastructure and
/// injects them into the presentation dependencies (composition root).
pub fn identity_dependencies(
    repository: Arc<identity_infrastructure::SeaOrmUserRepository>,
    hasher: identity_infrastructure::BoundedArgon2PasswordHasher,
    config: &Config,
    deletion: Arc<dyn identity_presentation::UserDeletionListener>,
    avatar_store: Arc<crate::avatar_store::FileAvatarStore>,
) -> SharedDependencies {
    let dummy_password_hash = hasher.dummy_password_hash();
    // One concrete repository, coerced to each port it implements before the
    // broader binding shadows the concrete name.
    let repository: Arc<identity_infrastructure::SeaOrmUserRepository> = repository;
    let throttle: Arc<dyn LoginThrottleRepository> = repository.clone();
    let repository: Arc<dyn UserRepository> = repository;
    let hasher: Arc<dyn PasswordHasher> = Arc::new(hasher);
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let admission: Arc<dyn LoginAdmission> =
        Arc::new(TokioSemaphoreAdmission::new(MAX_CONCURRENT_LOGINS));
    let avatar_store: Arc<dyn identity_application::ports::AvatarStore> = avatar_store;

    Arc::new(identity_presentation::IdentityDependencies {
        authenticate: Arc::new(
            identity_application::commands::authenticate::AuthenticateService::new(
                repository.clone(),
                throttle.clone(),
                hasher.clone(),
                clock.clone(),
                admission,
                dummy_password_hash.clone(),
            ),
        ),
        create_user: Arc::new(
            identity_application::commands::create_user::CreateUserService::new(
                repository.clone(),
                hasher.clone(),
                clock.clone(),
            ),
        ),
        change_password: Arc::new(
            identity_application::commands::change_password::ChangePasswordService::new(
                repository.clone(),
                hasher.clone(),
            ),
        ),
        delete_own_account: Arc::new(
            identity_application::commands::delete_own_account::DeleteOwnAccountService::new(
                repository.clone(),
            ),
        ),
        delete_user_as_admin: Arc::new(
            identity_application::commands::delete_user_as_admin::DeleteUserAsAdminService::new(
                repository.clone(),
            ),
        ),
        set_display_name: Arc::new(
            identity_application::commands::set_display_name::SetDisplayNameService::new(
                repository.clone(),
            ),
        ),
        set_user_status: Arc::new(
            identity_application::commands::set_user_status::SetUserStatusService::new(
                repository.clone(),
            ),
        ),
        setup_admin: Arc::new(
            identity_application::commands::setup_admin::SetupAdminService::new(
                repository.clone(),
                hasher.clone(),
                clock.clone(),
            ),
        ),
        clear_login_throttle: Arc::new(
            identity_application::commands::clear_login_throttle::ClearLoginThrottleService::new(
                throttle,
            ),
        ),
        replace_avatar: Arc::new(
            identity_application::commands::replace_avatar::ReplaceAvatarService::new(
                repository.clone(),
                avatar_store.clone(),
            ),
        ),
        clear_avatar: Arc::new(
            identity_application::commands::clear_avatar::ClearAvatarService::new(
                repository.clone(),
                avatar_store.clone(),
            ),
        ),
        read_avatar: Arc::new(
            identity_application::commands::read_avatar::ReadAvatarService::new(
                repository.clone(),
                avatar_store.clone(),
            ),
        ),
        count_users: Arc::new(
            identity_application::queries::count_users::CountUsersService::new(repository.clone()),
        ),
        find_user_by_id: Arc::new(
            identity_application::queries::find_user_by_id::FindUserByIdService::new(
                repository.clone(),
            ),
        ),
        list_users: Arc::new(
            identity_application::queries::list_users::ListUsersService::new(repository.clone()),
        ),
        setup_status: Arc::new(
            identity_application::queries::setup_status::SetupStatusService::new(repository),
        ),
        dummy_password_hash,
        session_secret: Arc::new(config.session_secret.expose().to_owned()),
        medioss_origin: Arc::new(config.medioss_origin.clone()),
        deletion,
        avatar_store,
    })
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

pub struct TokioSemaphoreAdmission {
    semaphore: Arc<tokio::sync::Semaphore>,
}

impl TokioSemaphoreAdmission {
    pub fn new(bound: usize) -> Self {
        Self {
            semaphore: Arc::new(tokio::sync::Semaphore::new(bound)),
        }
    }
}

const MAX_CONCURRENT_LOGINS: usize = 2;
// Hint returned to a login refused because every admission slot is taken.
const LOGIN_RETRY_AFTER_MILLIS: u64 = 250;

impl LoginAdmission for TokioSemaphoreAdmission {
    fn try_acquire(&self) -> Result<Box<dyn Send>, std::time::Duration> {
        // The owned permit IS the guard: holding it keeps the slot taken,
        // dropping it releases the slot, so MAX_CONCURRENT_LOGINS bounds
        // concurrent verifications.
        self.semaphore
            .clone()
            .try_acquire_owned()
            .map(|permit| Box::new(permit) as Box<dyn Send>)
            .map_err(|_| std::time::Duration::from_millis(LOGIN_RETRY_AFTER_MILLIS))
    }
}

#[cfg(test)]
#[path = "state.tests.rs"]
mod tests;
