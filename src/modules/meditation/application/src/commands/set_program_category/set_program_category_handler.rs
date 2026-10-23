use super::set_program_category::SetProgramCategory;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait SetProgramCategoryHandler: Send + Sync {
    async fn handle(&self, command: SetProgramCategory) -> Result<(), ApplicationError>;
}
