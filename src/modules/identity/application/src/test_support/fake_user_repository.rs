use async_trait::async_trait;
use identity_domain::{AvatarRef, DisplayName, Email, IdentityError, Role, User};
use kernel::UserId;

use super::{fake_repository::FakeRepository, fake_repository_state::FakeRepositoryState};
use crate::ports::{
    CredentialChange, InitialAdminRegistration, MemberRegistration, RepositoryError, UserRepository,
};

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

    async fn set_profile(
        &self,
        id: UserId,
        display_name: Option<DisplayName>,
        avatar: Option<AvatarRef>,
    ) -> Result<User, RepositoryError> {
        let mut state = self.state.lock().unwrap();
        let current = state
            .users
            .values()
            .find(|user| user.id() == id)
            .cloned()
            .ok_or(RepositoryError::UserNotFound)?;
        let changed = current
            .clone()
            .with_display_name(display_name)
            .with_avatar(avatar);
        let mut users = state.users.clone();
        users.insert(current.email().as_str().to_owned(), changed.clone());
        *state = FakeRepositoryState {
            users,
            ..state.clone()
        };
        Ok(changed)
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

fn map_credential_error(error: IdentityError) -> RepositoryError {
    match error {
        IdentityError::AuthVersionOverflow => RepositoryError::AuthVersionOverflow,
        _ => RepositoryError::OperationFailed,
    }
}
