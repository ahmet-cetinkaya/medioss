use kernel::UserId;
#[derive(Debug, Clone)]
pub struct RecordPosition {
    pub user_id: UserId,
    pub meditation_id: i64,
    pub position_ms: i64,
}

/// Bounded well above any real meditation's duration; rejects pathological
/// inputs before they can reach the progress-percent computation instead of
/// relying solely on wide-integer arithmetic there.
pub const MAX_POSITION_MS: i64 = 30 * 24 * 60 * 60 * 1000;
