use std::sync::Arc;
use std::time::Duration;

use crate::ports::LoginAdmission;

pub(crate) struct FakeLoginAdmission {
    retry_after: Option<Duration>,
}

impl FakeLoginAdmission {
    pub(crate) const fn available() -> Self {
        Self { retry_after: None }
    }

    pub(crate) const fn saturated(retry_after: Duration) -> Self {
        Self {
            retry_after: Some(retry_after),
        }
    }
}

impl LoginAdmission for FakeLoginAdmission {
    fn try_acquire(&self) -> Result<Arc<()>, Duration> {
        self.retry_after.map_or(Ok(Arc::new(())), Err)
    }
}
