//! Shared handler state.

use std::sync::Arc;

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
    pub identity: Arc<IdentityServices>,
    pub identity_dependencies: SharedDependencies<
        identity_infrastructure::SeaOrmUserRepository,
        identity_infrastructure::BoundedArgon2PasswordHasher,
        SystemClock,
        TokioSemaphoreAdmission,
    >,
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
            identity: Arc::clone(&self.identity),
            identity_dependencies: Arc::clone(&self.identity_dependencies),
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
        let dummy_password_hash = hasher.dummy_password_hash();
        let identity_dependencies: SharedDependencies<
            identity_infrastructure::SeaOrmUserRepository,
            identity_infrastructure::BoundedArgon2PasswordHasher,
            SystemClock,
            TokioSemaphoreAdmission,
        > = Arc::new(identity_presentation::IdentityDependencies {
            repository: Arc::clone(&repository),
            hasher: Arc::new(hasher),
            clock: Arc::new(SystemClock),
            admission: Arc::new(TokioSemaphoreAdmission::new(MAX_CONCURRENT_LOGINS)),
            dummy_password_hash,
            session_secret: Arc::new(config.session_secret.expose().to_owned()),
            medioss_origin: Arc::new(config.medioss_origin.clone()),
            deletion,
            avatar_store,
        });
        Self {
            config,
            meditation,
            mood_tracker,
            soundscape,
            portability,
            session_store,
            identity: Arc::new(IdentityServices { repository }),
            identity_dependencies,
        }
    }
}

pub struct IdentityServices {
    pub repository: Arc<identity_infrastructure::SeaOrmUserRepository>,
}

impl IdentityServices {
    pub async fn find_user_by_id(
        &self,
        id: kernel::UserId,
    ) -> Result<Option<identity_domain::User>, identity_application::ApplicationError> {
        identity_application::queries::find_user_by_id(self.repository.as_ref(), id).await
    }
}

pub struct SystemClock;

impl identity_application::ports::Clock for SystemClock {
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

impl identity_application::ports::LoginAdmission for TokioSemaphoreAdmission {
    type Permit = tokio::sync::OwnedSemaphorePermit;

    fn try_acquire(&self) -> Result<Self::Permit, std::time::Duration> {
        let semaphore = Arc::clone(&self.semaphore);
        semaphore
            .try_acquire_owned()
            .map_err(|_| std::time::Duration::from_millis(LOGIN_RETRY_AFTER_MILLIS))
    }
}
