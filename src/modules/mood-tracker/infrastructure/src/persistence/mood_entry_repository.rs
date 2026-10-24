use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::UserId;
use mood_tracker_application::ports::MoodEntryRepository;
use mood_tracker_domain::{MoodEntry, MoodScore};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

use crate::persistence::entities as table;

pub struct SeaOrmMoodEntryRepository {
    db: DatabaseConnection,
}

impl SeaOrmMoodEntryRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn to_domain(model: table::Model) -> Result<MoodEntry, String> {
    let score = MoodScore::new(model.score as u8).map_err(|error| error.to_string())?;
    let recorded_at = DateTime::parse_from_rfc3339(&model.recorded_at)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| format!("corrupt recorded_at {error}"))?;
    let updated_at = DateTime::parse_from_rfc3339(&model.updated_at)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| format!("corrupt updated_at {error}"))?;
    Ok(MoodEntry::new(
        model.id,
        UserId::new(model.user_id),
        score,
        model.note,
        recorded_at,
        updated_at,
    ))
}

#[async_trait]
impl MoodEntryRepository for SeaOrmMoodEntryRepository {
    async fn insert(
        &self,
        user_id: UserId,
        score: MoodScore,
        note: Option<String>,
        recorded_at: DateTime<Utc>,
    ) -> Result<MoodEntry, String> {
        let stamp = recorded_at.to_rfc3339();
        table::ActiveModel {
            user_id: Set(user_id.value()),
            score: Set(i32::from(score.value())),
            note: Set(note),
            recorded_at: Set(stamp.clone()),
            updated_at: Set(stamp),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(|error| error.to_string())
        .and_then(to_domain)
    }

    async fn update(
        &self,
        user_id: UserId,
        entry_id: i64,
        score: MoodScore,
        note: Option<String>,
        updated_at: DateTime<Utc>,
    ) -> Result<Option<MoodEntry>, String> {
        let Some(model) = table::Entity::find_by_id(entry_id)
            .filter(table::Column::UserId.eq(user_id.value()))
            .one(&self.db)
            .await
            .map_err(|error| error.to_string())?
        else {
            return Ok(None);
        };
        let mut active: table::ActiveModel = model.into();
        active.score = Set(i32::from(score.value()));
        active.note = Set(note);
        active.updated_at = Set(updated_at.to_rfc3339());
        active
            .update(&self.db)
            .await
            .map_err(|error| error.to_string())
            .and_then(to_domain)
            .map(Some)
    }

    async fn delete(&self, user_id: UserId, entry_id: i64) -> Result<bool, String> {
        Ok(table::Entity::delete_by_id(entry_id)
            .filter(table::Column::UserId.eq(user_id.value()))
            .exec(&self.db)
            .await
            .map_err(|error| error.to_string())?
            .rows_affected
            == 1)
    }

    async fn list_all_for_user(&self, user_id: UserId) -> Result<Vec<MoodEntry>, String> {
        table::Entity::find()
            .filter(table::Column::UserId.eq(user_id.value()))
            .order_by_desc(table::Column::RecordedAt)
            .order_by_desc(table::Column::Id)
            .all(&self.db)
            .await
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(to_domain)
            .collect()
    }

    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, String> {
        let rows = table::Entity::delete_many()
            .filter(table::Column::UserId.eq(user_id.value()))
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected;
        Ok(rows)
    }
}

#[cfg(test)]
#[path = "mood_entry_repository.tests.rs"]
mod tests;
