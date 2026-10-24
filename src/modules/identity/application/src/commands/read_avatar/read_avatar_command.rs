use kernel::UserId;

/// Reads one user's avatar bytes. Owner or admin only.
#[derive(Debug, Clone)]
pub struct ReadAvatarCommand {
    pub requested: UserId,
    pub reader: kernel::Actor,
}
