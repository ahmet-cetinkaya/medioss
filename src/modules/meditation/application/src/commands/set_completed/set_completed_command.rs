use kernel::UserId;
#[derive(Debug, Clone)]
pub struct SetCompletedCommand {
    pub user_id: UserId,
    pub meditation_id: i64,
    pub completed: bool,
}
