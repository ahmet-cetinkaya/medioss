use super::update_program_order::UpdateProgramOrder;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait UpdateProgramOrderHandler: Send + Sync {
    async fn handle(&self, command: UpdateProgramOrder) -> Result<(), ApplicationError>;
}
