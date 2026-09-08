use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use identity_domain::{Email, IdentityError, Role, User};
use kernel::UserId;

use super::{email, password, FakeHasher, FixedClock};
use crate::{
    ports::{
        CredentialChange, InitialAdminRegistration, MemberRegistration, RepositoryError,
        UserRepository,
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
            failed_attempts.insert(email.to_owned(), attempts);
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

    pub(crate) fn user_by_email(&self, email: &str) -> User {
        self.state.lock().unwrap().users[email].clone()
    }

    pub(crate) fn failed_attempts(&self, email: &str) -> u64 {
        self.state
            .lock()
            .unwrap()
            .failed_attempts
            .get(email)
            .copied()
            .unwrap_or_default()
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

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .users
            .get(email.as_str())
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
        let changed = current
            .clone()
            .replace_with_permanent_password(credential_change.password_hash().clone())
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

    async fn increment_failed_attempt(
        &self,
        normalized_email: &str,
        _attempted_at: DateTime<Utc>,
    ) -> Result<(), RepositoryError> {
        self.replace_state(|state| {
            let mut failed_attempts = state.failed_attempts.clone();
            let attempts = failed_attempts
                .get(normalized_email)
                .copied()
                .unwrap_or_default();
            failed_attempts.insert(normalized_email.to_owned(), attempts + 1);
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
