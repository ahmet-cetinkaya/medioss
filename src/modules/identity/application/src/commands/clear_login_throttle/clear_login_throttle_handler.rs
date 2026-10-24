use super::clear_login_throttle::ClearLoginThrottle;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait ClearLoginThrottleHandler: Send + Sync {
    async fn handle(&self, command: ClearLoginThrottle) -> Result<(), ApplicationError>;
}
