use kernel::UserId;

#[derive(Debug, Clone)]
pub struct DeleteMoodEntryCommand {
    pub user_id: UserId,
    pub entry_id: i64,
}
