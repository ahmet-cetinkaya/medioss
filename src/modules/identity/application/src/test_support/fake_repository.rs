use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use identity_domain::{Email, FailedAttempts, IdentityError, Role, User};
use kernel::UserId;

use super::{email, password, FakeHasher, FixedClock};
use crate::{
    ports::{
        CredentialChange, InitialAdminRegistration, LoginThrottleRepository, MemberRegistration,
        RepositoryError, UserRepository,
    },
    test_support::fake_repository_state::FakeRepositoryState,
};

pub(crate) struct FakeRepository {
    state: Mutex<FakeRepositoryState>,
}

impl FakeRepository {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(FakeRepositoryState {
                next_user_id: 1,
                ..FakeRepositoryState::default()
            }),
        }
    }

    pub(crate) fn with_permanent_admin() -> Self {
        let repository = Self::new();
        repository.seed_user(User::new_permanent(
            UserId::new(1),
            email(),
            FakeHasher::hash_for(&password()),
            Role::Admin,
            FixedClock::new().now_value(),
        ));
        repository
    }

    pub(crate) fn seed_user(&self, user: User) {
        self.replace_state(|state| {
            let mut users = state.users.clone();
            users.insert(user.email().as_str().to_owned(), user);
            FakeRepositoryState { users, ..state }
        });
    }

    pub(crate) fn set_failed_attempts(&self, email: &str, attempts: u64) {
        self.replace_state(|state| {
            let mut failed_attempts = state.failed_attempts.clone();
            let now = FixedClock::new().now_value();
            let blocked_until = now + chrono::TimeDelta::seconds(1);
            failed_attempts.insert(
                email.to_owned(),
                FailedAttempts::restore(attempts, now, blocked_until).unwrap(),
            );
            FakeRepositoryState {
                failed_attempts,
                ..state
            }
        });
    }

    pub(crate) fn fail_next_credential_change(&self) {
        self.replace_state(|state| FakeRepositoryState {
            should_fail_credential_change: true,
            ..state
        });
    }

    pub(crate) fn user_count(&self) -> usize {
        self.state.lock().unwrap().users.len()
    }

    pub(crate) fn user_by_id(&self, id: UserId) -> Option<User> {
        self.state
            .lock()
            .unwrap()
            .users
            .values()
            .find(|user| user.id() == id)
            .cloned()
    }

    pub(crate) fn user_by_email(&self, email: &str) -> User {
        self.state.lock().unwrap().users[email].clone()
    }

    pub(crate) fn failed_attempts(&self, email: &str) -> u64 {
        self.state
            .lock()
            .unwrap()
            .failed_attempts
            .get(email)
            .map_or(0, FailedAttempts::count)
    }

    pub(crate) fn count_calls(&self) -> usize {
        self.state.lock().unwrap().count_calls
    }

    pub(crate) fn initial_admin_insert_calls(&self) -> usize {
        self.state.lock().unwrap().initial_admin_insert_calls
    }

    pub(crate) fn credential_change_calls(&self) -> usize {
        self.state.lock().unwrap().credential_change_calls
    }

    fn replace_state(&self, update: impl FnOnce(FakeRepositoryState) -> FakeRepositoryState) {
        let mut state = self.state.lock().unwrap();
        *state = update(state.clone());
    }
}

#[async_trait]
impl UserRepository for FakeRepository {
    async fn count(&self) -> Result<u64, RepositoryError> {
        let mut state = self.state.lock().unwrap();
        let count = state.users.len() as u64;
        *state = FakeRepositoryState {
            count_calls: state.count_calls + 1,
            ..state.clone()
        };
        Ok(count)
    }

    async fn list(&self) -> Result<Vec<User>, RepositoryError> {
        let state = self.state.lock().unwrap();
        let mut users: Vec<User> = state.users.values().cloned().collect();
        users.sort_by(|a, b| a.email().as_str().cmp(b.email().as_str()));
        Ok(users)
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .users
            .get(email.as_str())
            .cloned())
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .users
            .values()
            .find(|user| user.id() == id)
            .cloned())
    }

    async fn insert_initial_admin_if_uninitialized(
        &self,
        registration: InitialAdminRegistration,
    ) -> Result<User, RepositoryError> {
        let mut state = self.state.lock().unwrap();
        let next_state = FakeRepositoryState {
            initial_admin_insert_calls: state.initial_admin_insert_calls + 1,
            ..state.clone()
        };

        if !state.users.is_empty() {
            *state = next_state;
            return Err(RepositoryError::SetupAlreadyCompleted);
        }

        let user = User::new_permanent(
            UserId::new(state.next_user_id),
            registration.email().clone(),
            registration.password_hash().clone(),
            Role::Admin,
            registration.created_at(),
        );
        let mut users = next_state.users.clone();
        users.insert(user.email().as_str().to_owned(), user.clone());
        *state = FakeRepositoryState {
            users,
            next_user_id: next_state.next_user_id + 1,
            ..next_state
        };
        Ok(user)
    }

    async fn insert_member(
        &self,
        registration: MemberRegistration,
    ) -> Result<User, RepositoryError> {
        let mut state = self.state.lock().unwrap();
        if state.users.contains_key(registration.email().as_str()) {
            return Err(RepositoryError::EmailAlreadyExists);
        }

        let user = User::new_temporary(
            UserId::new(state.next_user_id),
            registration.email().clone(),
            registration.password_hash().clone(),
            Role::Member,
            registration.created_at(),
        )
        .map_err(|_| RepositoryError::OperationFailed)?;
        let mut users = state.users.clone();
        users.insert(user.email().as_str().to_owned(), user.clone());
        *state = FakeRepositoryState {
            users,
            next_user_id: state.next_user_id + 1,
            ..state.clone()
        };
        Ok(user)
    }

    async fn apply_credential_change(
        &self,
        credential_change: CredentialChange,
    ) -> Result<User, RepositoryError> {
        let mut state = self.state.lock().unwrap();
        let current = state
            .users
            .values()
            .find(|user| user.id() == credential_change.user_id())
            .cloned()
            .ok_or(RepositoryError::UserNotFound)?;
        let changed = match credential_change.expires_at() {
            None => current
                .clone()
                .replace_with_permanent_password(credential_change.password_hash().clone()),
            Some(expires_at) => current.clone().replace_with_temporary_password(
                credential_change.password_hash().clone(),
                expires_at,
            ),
        }
        .map_err(map_credential_error)?;
        let failed_email = current.email().as_str().to_owned();
        let next_state = FakeRepositoryState {
            credential_change_calls: state.credential_change_calls + 1,
            should_fail_credential_change: false,
            ..state.clone()
        };

        if state.should_fail_credential_change {
            *state = next_state;
            return Err(RepositoryError::OperationFailed);
        }

        let mut users = next_state.users.clone();
        users.insert(failed_email.clone(), changed.clone());
        let mut failed_attempts = next_state.failed_attempts.clone();
        failed_attempts.remove(&failed_email);
        *state = FakeRepositoryState {
            users,
            failed_attempts,
            ..next_state
        };
        Ok(changed)
    }

    async fn set_disabled(&self, id: UserId, disabled: bool) -> Result<(), RepositoryError> {
        let mut state = self.state.lock().unwrap();
        let current = state
            .users
            .values()
            .find(|user| user.id() == id)
            .cloned()
            .ok_or(RepositoryError::UserNotFound)?;
        let mut users = state.users.clone();
        users.insert(
            current.email().as_str().to_owned(),
            current.with_disabled(disabled),
        );
        *state = FakeRepositoryState {
            users,
            ..state.clone()
        };
        Ok(())
    }

    async fn delete_user(&self, id: UserId) -> Result<bool, RepositoryError> {
        let mut state = self.state.lock().unwrap();
        let Some(current) = state.users.values().find(|user| user.id() == id).cloned() else {
            return Ok(false);
        };
        let mut users = state.users.clone();
        users.remove(current.email().as_str());
        *state = FakeRepositoryState {
            users,
            ..state.clone()
        };
        Ok(true)
    }

    async fn count_active_admins(&self) -> Result<u64, RepositoryError> {
        Ok(u64::try_from(
            self.state
                .lock()
                .unwrap()
                .users
                .values()
                .filter(|user| !user.is_disabled() && matches!(user.role(), Role::Admin))
                .count(),
        )
        .unwrap_or_default())
    }

    async fn delete_login_attempts(&self, normalized_email: &str) -> Result<(), RepositoryError> {
        let mut state = self.state.lock().unwrap();
        let mut failed_attempts = state.failed_attempts.clone();
        failed_attempts.remove(normalized_email);
        *state = FakeRepositoryState {
            failed_attempts,
            ..state.clone()
        };
        Ok(())
    }
}

#[async_trait]
impl LoginThrottleRepository for FakeRepository {
    async fn failed_attempts(
        &self,
        normalized_identifier: &str,
    ) -> Result<Option<FailedAttempts>, RepositoryError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .failed_attempts
            .get(normalized_identifier)
            .cloned())
    }

    async fn record_failed_attempt(
        &self,
        normalized_identifier: &str,
        attempted_at: DateTime<Utc>,
    ) -> Result<FailedAttempts, RepositoryError> {
        let mut state = self.state.lock().unwrap();
        let attempts = state
            .failed_attempts
            .get(normalized_identifier)
            .map_or_else(
                || FailedAttempts::first_failure(attempted_at),
                |current| current.record_failure(attempted_at),
            );
        let mut failed_attempts = state.failed_attempts.clone();
        failed_attempts.insert(normalized_identifier.to_owned(), attempts.clone());
        *state = FakeRepositoryState {
            failed_attempts,
            ..state.clone()
        };
        Ok(attempts)
    }

    async fn clear_failed_attempts(
        &self,
        normalized_identifier: &str,
    ) -> Result<(), RepositoryError> {
        self.replace_state(|state| {
            let mut failed_attempts = state.failed_attempts.clone();
            failed_attempts.remove(normalized_identifier);
            FakeRepositoryState {
                failed_attempts,
                ..state
            }
        });
        Ok(())
    }
}

fn map_credential_error(error: IdentityError) -> RepositoryError {
    match error {
        IdentityError::AuthVersionOverflow => RepositoryError::AuthVersionOverflow,
        _ => RepositoryError::OperationFailed,
    }
}
