use kernel::UserId;

#[derive(Debug, Clone)]
pub struct DeleteMix {
    pub id: i64,
    pub user_id: UserId,
}
