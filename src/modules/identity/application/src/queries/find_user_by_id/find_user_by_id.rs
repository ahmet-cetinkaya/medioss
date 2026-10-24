use kernel::UserId;

/// Resolves one account by id. `None` for an unknown id; the session
/// middleware uses this to admit or drop a request's actor.
#[derive(Debug, Clone)]
pub struct FindUserById {
    pub id: UserId,
}
