use kernel::Actor;

/// Clears the actor's avatar and unlinks its blob. Idempotent: no avatar
/// means nothing to do.
#[derive(Debug, Clone)]
pub struct ClearAvatarCommand {
    pub actor: Actor,
}
