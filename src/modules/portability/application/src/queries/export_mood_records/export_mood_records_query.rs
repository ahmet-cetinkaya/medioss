use kernel::UserId;

#[derive(Debug)]
pub struct ExportMoodRecordsQuery {
    pub user_id: UserId,
}
