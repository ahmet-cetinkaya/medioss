use kernel::UserId;

#[derive(Debug, Clone)]
pub struct GetProgramCover {
    /// Authenticated requester; covers are only served to signed-in users.
    #[allow(dead_code)]
    pub requester_id: UserId,
    pub program_id: i64,
}
