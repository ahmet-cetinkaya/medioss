use std::sync::Arc;

use super::clear_login_throttle_command::ClearLoginThrottleCommand;
use super::clear_login_throttle_command_validator::ClearLoginThrottleCommandValidator;
use crate::ports::CommandHandler;
use crate::ports::LoginThrottleRepository;
use crate::ApplicationError;

pub struct ClearLoginThrottleCommandHandler {
    validator: ClearLoginThrottleCommandValidator,
    throttle: Arc<dyn LoginThrottleRepository>,
}

impl ClearLoginThrottleCommandHandler {
    pub fn new(throttle: Arc<dyn LoginThrottleRepository>) -> Self {
        Self {
            validator: Default::default(),
            throttle,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<ClearLoginThrottleCommand> for ClearLoginThrottleCommandHandler {
    type Output = ();

    async fn handle(&self, command: ClearLoginThrottleCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;
        self.throttle
            .clear_failed_attempts(&self.validator.normalized_identifier(&command))
            .await
            .map_err(ApplicationError::from)
    }
}

#[cfg(test)]
#[path = "clear_login_throttle_command_handler.tests.rs"]
mod tests;
