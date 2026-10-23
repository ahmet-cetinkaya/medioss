use std::sync::Arc;

use super::delete_mix::DeleteMix;
use super::delete_mix_handler::DeleteMixHandler;
use crate::ports::MixRepository;
use crate::ApplicationError;

pub struct DeleteMixService {
    mixes: Arc<dyn MixRepository>,
}

impl DeleteMixService {
    pub fn new(mixes: Arc<dyn MixRepository>) -> Self {
        Self { mixes }
    }
}

#[async_trait::async_trait]
impl DeleteMixHandler for DeleteMixService {
    async fn handle(&self, command: DeleteMix) -> Result<(), ApplicationError> {
        if self
            .mixes
            .delete_for(command.id, command.user_id)
            .await
            .map_err(ApplicationError::Repository)?
        {
            Ok(())
        } else {
            Err(ApplicationError::NotFound)
        }
    }
}
