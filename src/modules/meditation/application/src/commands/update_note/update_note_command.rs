use kernel::UserId;
#[derive(Debug, Clone)]
pub struct UpdateNoteCommand {
    pub note_id: i64,
    pub meditation_id: i64,
    pub user_id: UserId,
    pub text: String,
}
