use kernel::UserId;

#[derive(Debug, Clone)]
pub struct CreateMoodEntryCommand {
    pub user_id: UserId,
    pub score: u8,
    pub note: Option<String>,
}
