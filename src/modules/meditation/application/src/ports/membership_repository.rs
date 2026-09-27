use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// A meditation's membership in a program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Membership {
    pub program_id: i64,
    pub meditation_id: i64,
    pub added_at: DateTime<Utc>,
}

/// Persists program memberships.
#[async_trait]
pub trait MembershipRepository: Send + Sync {
    /// Adds a meditation to a program. Idempotent on the (program, meditation)
    /// pair: a repeat add returns the existing membership's row id.
    async fn add(
        &self,
        program_id: i64,
        meditation_id: i64,
        added_at: DateTime<Utc>,
    ) -> Result<i64, String>;

    /// Removes a membership; `false` when the pair is not a member.
    async fn remove(&self, program_id: i64, meditation_id: i64) -> Result<bool, String>;

    /// Lists a program's meditations, ordered by `added_at` then row id.
    async fn list_by_program(&self, program_id: i64) -> Result<Vec<Membership>, String>;

    /// Removes every program membership of a meditation; returns the number
    /// removed.
    async fn remove_for_meditation(&self, meditation_id: i64) -> Result<u64, String>;

    /// The meditation's first (earliest `added_at`) program membership —
    /// its home program, used to resume "continue this program" links.
    async fn first_program_for_meditation(&self, meditation_id: i64)
        -> Result<Option<i64>, String>;
}
