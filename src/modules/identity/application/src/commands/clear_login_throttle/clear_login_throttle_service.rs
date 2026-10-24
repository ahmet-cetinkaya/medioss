use std::sync::Arc;

use super::clear_login_throttle::ClearLoginThrottle;
use super::clear_login_throttle_handler::ClearLoginThrottleHandler;
use crate::ports::LoginThrottleRepository;
use crate::ApplicationError;

pub struct ClearLoginThrottleService {
    throttle: Arc<dyn LoginThrottleRepository>,
}

impl ClearLoginThrottleService {
    pub fn new(throttle: Arc<dyn LoginThrottleRepository>) -> Self {
        Self { throttle }
    }
}

#[async_trait::async_trait]
impl ClearLoginThrottleHandler for ClearLoginThrottleService {
    async fn handle(&self, command: ClearLoginThrottle) -> Result<(), ApplicationError> {
        self.throttle
            .clear_failed_attempts(&command.identifier.trim().to_lowercase())
            .await
            .map_err(ApplicationError::from)
    }
}

#[cfg(test)]
#[path = "clear_login_throttle_service.tests.rs"]
mod tests;
