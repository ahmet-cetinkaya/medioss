use kernel::UserId;

#[derive(Debug, Clone)]
pub struct DeleteMoodEntry {
    pub user_id: UserId,
    pub entry_id: i64,
}
