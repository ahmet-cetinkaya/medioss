use chrono::{DateTime, Utc};
use kernel::UserId;
#[derive(Debug, Clone)]
pub struct RecordCompletion {
    pub user_id: UserId,
    pub meditation_id: i64,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}
