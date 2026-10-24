use kernel::UserId;

#[derive(Debug, Clone)]
pub struct UpdateMoodEntryCommand {
    pub user_id: UserId,
    pub entry_id: i64,
    pub score: u8,
    pub note: Option<String>,
}
