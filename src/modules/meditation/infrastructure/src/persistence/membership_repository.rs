//! `MembershipRepository` over SeaORM/SQLite.
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use meditation_application::ports::{Membership, MembershipRepository};
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
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
        position: model.position,
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
        let position = table::Entity::find()
            .filter(table::Column::ProgramId.eq(program_id))
            .order_by_desc(table::Column::Position)
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .map_or(0, |last| last.position + 1);
        let row = table::ActiveModel {
            program_id: Set(program_id),
            meditation_id: Set(meditation_id),
            added_at: Set(added_at.to_rfc3339()),
            position: Set(position),
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
            .order_by_asc(table::Column::Position)
            .order_by_asc(table::Column::Id)
            .all(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(to_domain)
            .collect()
    }

    async fn set_order(&self, program_id: i64, meditation_ids: &[i64]) -> Result<(), String> {
        let txn = self.db.begin().await.map_err(|e| e.to_string())?;
        for (index, meditation_id) in meditation_ids.iter().enumerate() {
            table::Entity::update_many()
                .col_expr(table::Column::Position, Expr::value(index as i64))
                .filter(table::Column::ProgramId.eq(program_id))
                .filter(table::Column::MeditationId.eq(*meditation_id))
                .exec(&txn)
                .await
                .map_err(|e| e.to_string())?;
        }
        txn.commit().await.map_err(|e| e.to_string())
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

    async fn home_programs(&self) -> Result<std::collections::HashMap<i64, (i64, bool)>, String> {
        // One pass over all memberships (earliest added_at, then row id, wins
        // per meditation), joined with the program's cover presence.
        let rows = self
            .db
            .query_all(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT m.meditation_id AS meditation_id, m.program_id AS program_id, \
                 (p.cover_storage_key IS NOT NULL) AS has_cover \
                 FROM meditation_program_memberships m \
                 JOIN meditation_programs p ON p.id = m.program_id \
                 ORDER BY m.added_at ASC, m.id ASC"
                    .to_owned(),
            ))
            .await
            .map_err(|e| e.to_string())?;
        let mut homes = std::collections::HashMap::new();
        for row in rows {
            let meditation_id: i64 = row
                .try_get("", "meditation_id")
                .map_err(|e| e.to_string())?;
            // insert keeps the first row seen for a meditation; the ORDER BY
            // makes that the earliest membership.
            homes.entry(meditation_id).or_insert((
                row.try_get::<i64>("", "program_id")
                    .map_err(|e| e.to_string())?,
                row.try_get::<i64>("", "has_cover")
                    .map_err(|e| e.to_string())?
                    != 0,
            ));
        }
        Ok(homes)
    }
}

#[cfg(test)]
#[path = "membership_repository.tests.rs"]
mod tests;
