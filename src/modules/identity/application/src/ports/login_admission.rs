use std::time::Duration;

/// Bounds concurrent credential verifications.
///
/// `try_acquire` returns an opaque guard: holding it keeps the admission
/// slot taken; dropping it releases the slot. The guard is type-erased so
/// the port can be shared as `Arc<dyn LoginAdmission>` and so the
/// application layer does not name the concurrency primitive.
pub trait LoginAdmission: Send + Sync {
    fn try_acquire(&self) -> Result<Box<dyn Send>, Duration>;
}
