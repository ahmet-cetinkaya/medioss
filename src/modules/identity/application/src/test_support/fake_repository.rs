use std::sync::Mutex;

use identity_domain::{FailedAttempts, Role, User};
use kernel::UserId;

use super::fake_repository_state::FakeRepositoryState;
use super::{email, password, FakeHasher, FixedClock};

pub(crate) struct FakeRepository {
    pub(super) state: Mutex<FakeRepositoryState>,
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

    pub(super) fn replace_state(
        &self,
        update: impl FnOnce(FakeRepositoryState) -> FakeRepositoryState,
    ) {
        let mut state = self.state.lock().unwrap();
        *state = update(state.clone());
    }
}
