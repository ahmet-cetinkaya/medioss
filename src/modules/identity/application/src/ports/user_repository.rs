use async_trait::async_trait;
use identity_domain::{Email, User};
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
}
