//! `MembershipRepository` over SeaORM/SQLite.
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use meditation_application::ports::{Membership, MembershipRepository};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

use crate::persistence::entities::memberships as table;

pub struct SeaOrmMembershipRepository {
    db: DatabaseConnection,
}

impl SeaOrmMembershipRepository {
    /// Wraps a shared connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn to_domain(model: table::Model) -> Result<Membership, String> {
    let added_at = DateTime::parse_from_rfc3339(&model.added_at)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| format!("corrupt added_at {error}"))?;
    Ok(Membership {
        program_id: model.program_id,
        meditation_id: model.meditation_id,
        added_at,
    })
}

#[async_trait]
impl MembershipRepository for SeaOrmMembershipRepository {
    async fn add(
        &self,
        program_id: i64,
        meditation_id: i64,
        added_at: DateTime<Utc>,
    ) -> Result<i64, String> {
        let existing = table::Entity::find()
            .filter(table::Column::ProgramId.eq(program_id))
            .filter(table::Column::MeditationId.eq(meditation_id))
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        if let Some(model) = existing {
            return Ok(model.id);
        }
        let row = table::ActiveModel {
            program_id: Set(program_id),
            meditation_id: Set(meditation_id),
            added_at: Set(added_at.to_rfc3339()),
            ..Default::default()
        };
        match row.insert(&self.db).await {
            Ok(model) => Ok(model.id),
            // A concurrent insert may have hit the UNIQUE pair first: read
            // it back before surfacing the original error as a failure.
            Err(insert_error) => table::Entity::find()
                .filter(table::Column::ProgramId.eq(program_id))
                .filter(table::Column::MeditationId.eq(meditation_id))
                .one(&self.db)
                .await
                .map_err(|e| e.to_string())?
                .map(|model| model.id)
                .ok_or_else(|| insert_error.to_string()),
        }
    }

    async fn remove(&self, program_id: i64, meditation_id: i64) -> Result<bool, String> {
        let rows = table::Entity::delete_many()
            .filter(table::Column::ProgramId.eq(program_id))
            .filter(table::Column::MeditationId.eq(meditation_id))
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected;
        Ok(rows > 0)
    }

    async fn list_by_program(&self, program_id: i64) -> Result<Vec<Membership>, String> {
        table::Entity::find()
            .filter(table::Column::ProgramId.eq(program_id))
            .order_by_asc(table::Column::Id)
            .all(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(to_domain)
            .collect()
    }

    async fn remove_for_meditation(&self, meditation_id: i64) -> Result<u64, String> {
        let rows = table::Entity::delete_many()
            .filter(table::Column::MeditationId.eq(meditation_id))
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected;
        Ok(rows)
    }

    async fn first_program_for_meditation(
        &self,
        meditation_id: i64,
    ) -> Result<Option<i64>, String> {
        let model = table::Entity::find()
            .filter(table::Column::MeditationId.eq(meditation_id))
            .order_by_asc(table::Column::AddedAt)
            .order_by_asc(table::Column::Id)
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(model.map(|row| row.program_id))
    }
}
