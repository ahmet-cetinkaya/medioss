use async_trait::async_trait;
use identity_domain::{AvatarRef, DisplayName, Email, User};
use kernel::UserId;

use super::{CredentialChange, InitialAdminRegistration, MemberRegistration, RepositoryError};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn count(&self) -> Result<u64, RepositoryError>;

    async fn list(&self) -> Result<Vec<User>, RepositoryError>;

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError>;

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError>;

    async fn insert_initial_admin_if_uninitialized(
        &self,
        registration: InitialAdminRegistration,
    ) -> Result<User, RepositoryError>;

    async fn insert_member(
        &self,
        registration: MemberRegistration,
    ) -> Result<User, RepositoryError>;

    async fn apply_credential_change(
        &self,
        credential_change: CredentialChange,
    ) -> Result<User, RepositoryError>;

    async fn set_disabled(&self, id: UserId, disabled: bool) -> Result<(), RepositoryError>;

    async fn set_profile(
        &self,
        id: UserId,
        display_name: Option<DisplayName>,
        avatar: Option<AvatarRef>,
    ) -> Result<User, RepositoryError>;

    async fn delete_user(&self, id: UserId) -> Result<bool, RepositoryError>;

    async fn count_active_admins(&self) -> Result<u64, RepositoryError>;

    async fn delete_login_attempts(&self, normalized_email: &str) -> Result<(), RepositoryError>;
}
