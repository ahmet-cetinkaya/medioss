use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::UserId;
use mood_tracker_application::ports::{MoodEntryRepository, Page, PageRequest};
use mood_tracker_domain::{MoodEntry, MoodScore};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
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

    async fn find_owned(
        &self,
        user_id: UserId,
        entry_id: i64,
    ) -> Result<Option<MoodEntry>, String> {
        table::Entity::find_by_id(entry_id)
            .filter(table::Column::UserId.eq(user_id.value()))
            .one(&self.db)
            .await
            .map_err(|error| error.to_string())?
            .map(to_domain)
            .transpose()
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

    async fn list_for_user(
        &self,
        user_id: UserId,
        page: PageRequest,
    ) -> Result<Page<MoodEntry>, String> {
        let mut query = table::Entity::find().filter(table::Column::UserId.eq(user_id.value()));
        if let Some((at, id)) = page.cursor {
            query = query.filter(
                Condition::any()
                    .add(table::Column::RecordedAt.lt(at.to_rfc3339()))
                    .add(
                        Condition::all()
                            .add(table::Column::RecordedAt.eq(at.to_rfc3339()))
                            .add(table::Column::Id.lt(id)),
                    ),
            );
        }
        let mut models = query
            .order_by_desc(table::Column::RecordedAt)
            .order_by_desc(table::Column::Id)
            .limit(u64::from(page.limit) + 1)
            .all(&self.db)
            .await
            .map_err(|error| error.to_string())?;
        let has_more = models.len() > usize::from(page.limit);
        models.truncate(usize::from(page.limit));
        let items: Vec<_> = models
            .into_iter()
            .map(to_domain)
            .collect::<Result<_, _>>()?;
        let next_cursor = has_more.then(|| {
            let last = items
                .last()
                .expect("page has items when it has a continuation");
            (last.recorded_at(), last.id())
        });
        Ok(Page { items, next_cursor })
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
}
