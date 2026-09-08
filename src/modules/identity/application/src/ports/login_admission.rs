use std::time::Duration;

pub trait LoginAdmission: Send + Sync {
    type Permit: Send;

    fn try_acquire(&self) -> Result<Self::Permit, Duration>;
}
