use kernel::UserId;

#[derive(Debug, Clone)]
pub struct DeleteMixCommand {
    pub id: i64,
    pub user_id: UserId,
}
