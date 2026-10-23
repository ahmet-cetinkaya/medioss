use super::add_member::AddMember;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait AddMemberHandler: Send + Sync {
    async fn handle(&self, command: AddMember) -> Result<(), ApplicationError>;
}
