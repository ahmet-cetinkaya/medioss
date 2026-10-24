use kernel::UserId;
#[derive(Debug, Clone)]
pub struct DeleteNoteCommand {
    pub note_id: i64,
    pub meditation_id: i64,
    pub user_id: UserId,
}
