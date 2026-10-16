use std::sync::Arc;

use identity_application::ports::{
    Clock, LoginAdmission, LoginThrottleRepository, PasswordHasher, RepositoryError, UserRepository,
};
use identity_domain::{Email, FailedAttempts, PasswordHash, RawPassword, User};
use kernel::Role;

use crate::{identity_router, IdentityDependencies, SharedDependencies};

pub(super) struct NullHasher;

#[async_trait::async_trait]
impl PasswordHasher for NullHasher {
    async fn hash(
        &self,
        password: &RawPassword,
    ) -> Result<PasswordHash, identity_application::ports::PasswordHasherError> {
        Ok(PasswordHash::from_stored(format!(
            "null:{}",
            password.as_str()
        )))
    }

    async fn verify(
        &self,
        password: &str,
        password_hash: &PasswordHash,
    ) -> Result<bool, identity_application::ports::PasswordHasherError> {
        Ok(password_hash.as_str() == format!("null:{password}"))
    }
}

pub(super) struct NullClock;

impl Clock for NullClock {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

pub(super) struct NullAdmission;

impl LoginAdmission for NullAdmission {
    type Permit = ();

    fn try_acquire(&self) -> Result<Self::Permit, std::time::Duration> {
        Ok(())
    }
}

pub(super) struct RecordingAvatarStore {
    pub(super) normalize_result: std::sync::Mutex<
        Result<(Vec<u8>, &'static str), identity_application::ports::AvatarStoreError>,
    >,
    pub(super) written: std::sync::Mutex<Vec<&'static str>>,
    pub(super) removed: std::sync::Mutex<Vec<&'static str>>,
    pub(super) read_result: std::sync::Mutex<Vec<u8>>,
    pub(super) next_key: std::sync::atomic::AtomicU64,
}

impl RecordingAvatarStore {
    pub(super) fn accepting() -> Self {
        Self {
            normalize_result: std::sync::Mutex::new(Ok((vec![1, 2, 3], "webp"))),
            written: std::sync::Mutex::new(Vec::new()),
            removed: std::sync::Mutex::new(Vec::new()),
            read_result: std::sync::Mutex::new(vec![9, 9]),
            next_key: std::sync::atomic::AtomicU64::new(1),
        }
    }
}

#[async_trait::async_trait]
impl identity_application::ports::AvatarStore for RecordingAvatarStore {
    async fn normalize(
        &self,
        _bytes: Vec<u8>,
    ) -> Result<(Vec<u8>, &'static str), identity_application::ports::AvatarStoreError> {
        self.normalize_result.lock().unwrap().clone()
    }

    async fn write(
        &self,
        _id: kernel::UserId,
        extension: &'static str,
        _bytes: &[u8],
    ) -> Result<identity_domain::AvatarRef, identity_application::ports::AvatarStoreError> {
        let sequence = self
            .next_key
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.written.lock().unwrap().push(extension);
        Ok(identity_domain::AvatarRef::new(
            format!("avatar-{sequence:016x}.{extension}"),
            extension,
        )
        .unwrap())
    }

    async fn read(
        &self,
        _id: kernel::UserId,
        _avatar: &identity_domain::AvatarRef,
    ) -> Result<Vec<u8>, identity_application::ports::AvatarStoreError> {
        Ok(self.read_result.lock().unwrap().clone())
    }

    async fn remove(
        &self,
        _id: kernel::UserId,
        avatar: &identity_domain::AvatarRef,
    ) -> Result<(), identity_application::ports::AvatarStoreError> {
        self.removed.lock().unwrap().push(avatar.extension());
        Ok(())
    }
}

pub(super) struct NullAvatarStore;

#[async_trait::async_trait]
impl identity_application::ports::AvatarStore for NullAvatarStore {
    async fn normalize(
        &self,
        _bytes: Vec<u8>,
    ) -> Result<(Vec<u8>, &'static str), identity_application::ports::AvatarStoreError> {
        Err(identity_application::ports::AvatarStoreError::InvalidPayload)
    }

    async fn write(
        &self,
        _id: kernel::UserId,
        extension: &'static str,
        _bytes: &[u8],
    ) -> Result<identity_domain::AvatarRef, identity_application::ports::AvatarStoreError> {
        Ok(identity_domain::AvatarRef::from_extension(extension).unwrap())
    }

    async fn read(
        &self,
        _id: kernel::UserId,
        _avatar: &identity_domain::AvatarRef,
    ) -> Result<Vec<u8>, identity_application::ports::AvatarStoreError> {
        Ok(Vec::new())
    }

    async fn remove(
        &self,
        _id: kernel::UserId,
        _avatar: &identity_domain::AvatarRef,
    ) -> Result<(), identity_application::ports::AvatarStoreError> {
        Ok(())
    }
}

pub(super) struct EmptyRepository;

#[async_trait::async_trait]
impl UserRepository for EmptyRepository {
    async fn count(&self) -> Result<u64, RepositoryError> {
        Ok(0)
    }

    async fn list(&self) -> Result<Vec<User>, RepositoryError> {
        Ok(Vec::new())
    }

    async fn find_by_email(&self, _email: &Email) -> Result<Option<User>, RepositoryError> {
        Ok(None)
    }

    async fn find_by_id(&self, _id: kernel::UserId) -> Result<Option<User>, RepositoryError> {
        Ok(None)
    }

    async fn insert_initial_admin_if_uninitialized(
        &self,
        _registration: identity_application::ports::InitialAdminRegistration,
    ) -> Result<User, RepositoryError> {
        Err(RepositoryError::OperationFailed)
    }

    async fn insert_member(
        &self,
        _registration: identity_application::ports::MemberRegistration,
    ) -> Result<User, RepositoryError> {
        Err(RepositoryError::OperationFailed)
    }

    async fn apply_credential_change(
        &self,
        _credential_change: identity_application::ports::CredentialChange,
    ) -> Result<User, RepositoryError> {
        Err(RepositoryError::OperationFailed)
    }

    async fn set_disabled(
        &self,
        _id: kernel::UserId,
        _disabled: bool,
    ) -> Result<(), RepositoryError> {
        Err(RepositoryError::OperationFailed)
    }

    async fn set_profile(
        &self,
        _id: kernel::UserId,
        _display_name: Option<identity_domain::DisplayName>,
        _avatar: Option<identity_domain::AvatarRef>,
    ) -> Result<User, RepositoryError> {
        Err(RepositoryError::OperationFailed)
    }

    async fn delete_user(&self, _id: kernel::UserId) -> Result<bool, RepositoryError> {
        Ok(false)
    }

    async fn count_active_admins(&self) -> Result<u64, RepositoryError> {
        Ok(0)
    }

    async fn delete_login_attempts(&self, _normalized_email: &str) -> Result<(), RepositoryError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl LoginThrottleRepository for EmptyRepository {
    async fn failed_attempts(
        &self,
        _normalized_identifier: &str,
    ) -> Result<Option<FailedAttempts>, RepositoryError> {
        Ok(None)
    }

    async fn record_failed_attempt(
        &self,
        _normalized_identifier: &str,
        _attempted_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<FailedAttempts, RepositoryError> {
        // Reaching the throttle is legitimate for an unknown email (the
        // response is deliberately uniform); return a fresh zero-count
        // record instead of panicking.
        Ok(FailedAttempts::first_failure(_attempted_at))
    }

    async fn clear_failed_attempts(
        &self,
        _normalized_identifier: &str,
    ) -> Result<(), RepositoryError> {
        Ok(())
    }
}

pub(super) fn dependencies(
) -> SharedDependencies<EmptyRepository, NullHasher, NullClock, NullAdmission> {
    Arc::new(IdentityDependencies {
        repository: Arc::new(EmptyRepository),
        hasher: Arc::new(NullHasher),
        clock: Arc::new(NullClock),
        admission: Arc::new(NullAdmission),
        dummy_password_hash: PasswordHash::from_stored("null:dummy".to_owned()),
        session_secret: Arc::new("session-secret-value".to_owned()),
        medioss_origin: Arc::new("https://medioss.example".to_owned()),
        deletion: std::sync::Arc::new(crate::users_lifecycle::NoUserDeletion),
        avatar_store: std::sync::Arc::new(NullAvatarStore),
    })
}

pub(super) fn app() -> axum::Router {
    identity_router::<EmptyRepository, NullHasher, NullClock, NullAdmission>()
        .with_state(dependencies())
        .layer(tower_sessions::SessionManagerLayer::new(
            tower_sessions::MemoryStore::default(),
        ))
}

pub(super) struct SeededRepository {
    pub(super) inner: EmptyRepository,
    pub(super) users: std::sync::Mutex<Vec<User>>,
}

#[async_trait::async_trait]
impl UserRepository for SeededRepository {
    async fn count(&self) -> Result<u64, RepositoryError> {
        Ok(self.users.lock().unwrap().len() as u64)
    }

    async fn list(&self) -> Result<Vec<User>, RepositoryError> {
        let mut users = self.users.lock().unwrap().clone();
        users.sort_by(|a, b| a.email().as_str().cmp(b.email().as_str()));
        Ok(users)
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|user| user.email().as_str() == email.as_str())
            .cloned())
    }

    async fn find_by_id(&self, id: kernel::UserId) -> Result<Option<User>, RepositoryError> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|user| user.id() == id)
            .cloned())
    }

    async fn insert_initial_admin_if_uninitialized(
        &self,
        registration: identity_application::ports::InitialAdminRegistration,
    ) -> Result<User, RepositoryError> {
        self.inner
            .insert_initial_admin_if_uninitialized(registration)
            .await
    }

    async fn insert_member(
        &self,
        registration: identity_application::ports::MemberRegistration,
    ) -> Result<User, RepositoryError> {
        self.inner.insert_member(registration).await
    }

    async fn apply_credential_change(
        &self,
        credential_change: identity_application::ports::CredentialChange,
    ) -> Result<User, RepositoryError> {
        self.inner.apply_credential_change(credential_change).await
    }

    async fn set_disabled(
        &self,
        id: kernel::UserId,
        disabled: bool,
    ) -> Result<(), RepositoryError> {
        let mut users = self.users.lock().unwrap();
        let Some(current) = users.iter_mut().find(|user| user.id() == id) else {
            return Err(RepositoryError::UserNotFound);
        };
        *current = current.clone().with_disabled(disabled);
        Ok(())
    }

    async fn set_profile(
        &self,
        id: kernel::UserId,
        display_name: Option<identity_domain::DisplayName>,
        avatar: Option<identity_domain::AvatarRef>,
    ) -> Result<User, RepositoryError> {
        let mut users = self.users.lock().unwrap();
        let Some(current) = users.iter_mut().find(|user| user.id() == id) else {
            return Err(RepositoryError::UserNotFound);
        };
        *current = current
            .clone()
            .with_display_name(display_name)
            .with_avatar(avatar);
        Ok(current.clone())
    }

    async fn delete_user(&self, id: kernel::UserId) -> Result<bool, RepositoryError> {
        let mut users = self.users.lock().unwrap();
        let before = users.len();
        users.retain(|user| user.id() != id);
        Ok(users.len() != before)
    }

    async fn count_active_admins(&self) -> Result<u64, RepositoryError> {
        Ok(u64::try_from(
            self.users
                .lock()
                .unwrap()
                .iter()
                .filter(|user| !user.is_disabled() && matches!(user.role(), Role::Admin))
                .count(),
        )
        .unwrap_or_default())
    }

    async fn delete_login_attempts(&self, normalized_email: &str) -> Result<(), RepositoryError> {
        self.inner.delete_login_attempts(normalized_email).await
    }
}

#[async_trait::async_trait]
impl LoginThrottleRepository for SeededRepository {
    async fn failed_attempts(
        &self,
        normalized_identifier: &str,
    ) -> Result<Option<FailedAttempts>, RepositoryError> {
        self.inner.failed_attempts(normalized_identifier).await
    }

    async fn record_failed_attempt(
        &self,
        normalized_identifier: &str,
        attempted_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<FailedAttempts, RepositoryError> {
        self.inner
            .record_failed_attempt(normalized_identifier, attempted_at)
            .await
    }

    async fn clear_failed_attempts(
        &self,
        normalized_identifier: &str,
    ) -> Result<(), RepositoryError> {
        self.inner
            .clear_failed_attempts(normalized_identifier)
            .await
    }
}

pub(super) fn seeded_app(users: Vec<User>) -> axum::Router {
    seeded_app_with(users, Arc::new(NullAvatarStore))
}

pub(super) fn seeded_app_with(
    users: Vec<User>,
    avatar_store: Arc<dyn identity_application::ports::AvatarStore>,
) -> axum::Router {
    let dependencies = Arc::new(IdentityDependencies {
        repository: Arc::new(SeededRepository {
            inner: EmptyRepository,
            users: std::sync::Mutex::new(users),
        }),
        hasher: Arc::new(NullHasher),
        clock: Arc::new(NullClock),
        admission: Arc::new(NullAdmission),
        dummy_password_hash: PasswordHash::from_stored("null:dummy".to_owned()),
        session_secret: Arc::new("session-secret-value".to_owned()),
        medioss_origin: Arc::new("https://medioss.example".to_owned()),
        deletion: std::sync::Arc::new(crate::users_lifecycle::NoUserDeletion),
        avatar_store,
    });
    identity_router::<SeededRepository, NullHasher, NullClock, NullAdmission>()
        .with_state(dependencies)
        .layer(tower_sessions::SessionManagerLayer::new(
            tower_sessions::MemoryStore::default(),
        ))
}

pub(super) fn user(id: i64, email: &str, role: Role) -> User {
    User::new_permanent(
        kernel::UserId::new(id),
        Email::parse(email).unwrap(),
        PasswordHash::from_stored("null:x".to_owned()),
        role,
        chrono::Utc::now(),
    )
}

pub(super) fn disabled_user(id: i64, email: &str, role: Role) -> User {
    user(id, email, role).with_disabled(true)
}
