use kernel::Actor;

/// Stores a normalized avatar for the actor and unlinks the previous blob
/// only after the profile update commits.
#[derive(Debug, Clone)]
pub struct ReplaceAvatar {
    pub actor: Actor,
    pub bytes: Vec<u8>,
}
