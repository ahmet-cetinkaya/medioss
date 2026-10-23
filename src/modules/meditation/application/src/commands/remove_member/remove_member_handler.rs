use super::remove_member::RemoveMember;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait RemoveMemberHandler: Send + Sync {
    async fn handle(&self, command: RemoveMember) -> Result<(), ApplicationError>;
}
