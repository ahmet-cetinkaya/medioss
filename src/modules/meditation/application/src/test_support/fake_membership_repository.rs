use chrono::{DateTime, Utc};

use super::repo_state::RepoState;
use crate::ports::{Membership, MembershipRepository};

/// MembershipRepository fake.
#[derive(Clone)]
pub struct FakeMembershipRepository(pub RepoState);

#[async_trait::async_trait]
impl MembershipRepository for FakeMembershipRepository {
    async fn add(
        &self,
        program_id: i64,
        meditation_id: i64,
        added_at: DateTime<Utc>,
    ) -> Result<i64, String> {
        if *self.0.membership_add_failures.lock().unwrap() {
            return Err("membership insert failed".to_owned());
        }
        let mut rows = self.0.memberships.lock().unwrap();
        let existing_index = rows.iter().position(|membership| {
            membership.program_id == program_id && membership.meditation_id == meditation_id
        });
        if let Some(index) = existing_index {
            // Row id == 1-based position, matching the fresh-insert numbering
            // below.
            return Ok(index as i64 + 1);
        }
        let row_id = rows.len() as i64 + 1;
        let position = rows
            .iter()
            .filter(|membership| membership.program_id == program_id)
            .map(|membership| membership.position + 1)
            .max()
            .unwrap_or(0);
        rows.push(Membership {
            program_id,
            meditation_id,
            added_at,
            position,
        });
        Ok(row_id)
    }

    async fn remove(&self, program_id: i64, meditation_id: i64) -> Result<bool, String> {
        let mut rows = self.0.memberships.lock().unwrap();
        let len = rows.len();
        rows.retain(|membership| {
            !(membership.program_id == program_id && membership.meditation_id == meditation_id)
        });
        Ok(rows.len() != len)
    }

    async fn list_by_program(&self, program_id: i64) -> Result<Vec<Membership>, String> {
        let mut rows: Vec<Membership> = self
            .0
            .memberships
            .lock()
            .unwrap()
            .iter()
            .filter(|membership| membership.program_id == program_id)
            .cloned()
            .collect();
        rows.sort_by_key(|membership| membership.position);
        Ok(rows)
    }

    async fn set_order(&self, program_id: i64, meditation_ids: &[i64]) -> Result<(), String> {
        let mut rows = self.0.memberships.lock().unwrap();
        for (index, id) in meditation_ids.iter().enumerate() {
            if let Some(row) = rows
                .iter_mut()
                .find(|m| m.program_id == program_id && m.meditation_id == *id)
            {
                row.position = index as i64;
            }
        }
        Ok(())
    }

    async fn remove_for_meditation(&self, meditation_id: i64) -> Result<u64, String> {
        let mut rows = self.0.memberships.lock().unwrap();
        let len = rows.len();
        rows.retain(|membership| membership.meditation_id != meditation_id);
        Ok((len - rows.len()) as u64)
    }

    async fn first_program_for_meditation(
        &self,
        meditation_id: i64,
    ) -> Result<Option<i64>, String> {
        let mut rows: Vec<Membership> = self
            .0
            .memberships
            .lock()
            .unwrap()
            .iter()
            .filter(|membership| membership.meditation_id == meditation_id)
            .cloned()
            .collect();
        rows.sort_by_key(|membership| membership.added_at);
        Ok(rows.first().map(|membership| membership.program_id))
    }

    async fn home_programs(&self) -> Result<std::collections::HashMap<i64, (i64, bool)>, String> {
        let mut rows: Vec<Membership> = self.0.memberships.lock().unwrap().clone();
        rows.sort_by_key(|membership| membership.added_at);
        let covers: std::collections::HashMap<i64, bool> = self
            .0
            .programs
            .lock()
            .unwrap()
            .iter()
            .map(|program| (program.id, program.cover_storage_key.is_some()))
            .collect();
        let mut homes = std::collections::HashMap::new();
        for membership in rows {
            homes.entry(membership.meditation_id).or_insert_with(|| {
                let has_cover = covers.get(&membership.program_id).copied().unwrap_or(false);
                (membership.program_id, has_cover)
            });
        }
        Ok(homes)
    }
}
