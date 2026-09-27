//! A program's members in add order.
use crate::ports::{Membership, MembershipRepository};
use crate::ApplicationError;

/// Lists a program's memberships ordered by `added_at` (spec FR-027).
pub async fn list_program_members<M>(
    memberships: &M,
    program_id: i64,
) -> Result<Vec<Membership>, ApplicationError>
where
    M: MembershipRepository + ?Sized,
{
    memberships
        .list_by_program(program_id)
        .await
        .map_err(ApplicationError::Repository)
}
