use std::sync::Arc;
use std::time::Duration;

/// Bounds concurrent credential verifications. The permit is type-erased so
/// the admission port can be shared as `Arc<dyn LoginAdmission>`; holding it
/// is all the caller does, so erasure changes no behavior.
pub trait LoginAdmission: Send + Sync {
    fn try_acquire(&self) -> Result<Arc<()>, Duration>;
}
