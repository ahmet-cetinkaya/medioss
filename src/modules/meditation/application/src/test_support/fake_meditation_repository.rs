use chrono::{DateTime, Utc};

use super::repo_state::RepoState;

/// MeditationRepository fake.
#[derive(Clone)]
pub struct FakeMeditationRepository(pub RepoState);

#[async_trait::async_trait]
impl crate::ports::MeditationRepository for FakeMeditationRepository {
    async fn insert(
        &self,
        name: String,
        content_type: &str,
        size_bytes: i64,
        storage_key: String,
        duration_ms: i64,
        created_at: DateTime<Utc>,
        description: Option<String>,
    ) -> Result<meditation_domain::Meditation, String> {
        let mut rows = self.0.meditations.lock().unwrap();
        let id = RepoState::next_id(&rows, |meditation| meditation.id());
        let meditation = meditation_domain::Meditation::new(
            id,
            &name,
            content_type,
            size_bytes,
            &storage_key,
            duration_ms,
            created_at,
        )
        .and_then(|row| row.with_description(description.as_deref()))
        .map_err(|error| error.to_string())?;
        rows.push(meditation.clone());
        Ok(meditation)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<meditation_domain::Meditation>, String> {
        Ok(self
            .0
            .meditations
            .lock()
            .unwrap()
            .iter()
            .find(|meditation| meditation.id() == id)
            .cloned())
    }

    async fn list_all(&self) -> Result<Vec<meditation_domain::Meditation>, String> {
        let mut rows = self.0.meditations.lock().unwrap().clone();
        rows.sort_by(|left, right| {
            right
                .created_at()
                .cmp(&left.created_at())
                .then_with(|| right.id().cmp(&left.id()))
        });
        Ok(rows)
    }

    async fn delete(&self, id: i64) -> Result<bool, String> {
        let mut rows = self.0.meditations.lock().unwrap();
        let len = rows.len();
        rows.retain(|meditation| meditation.id() != id);
        Ok(rows.len() != len)
    }

    async fn list_by_storage_prefix(
        &self,
        prefix: &str,
    ) -> Result<Vec<meditation_domain::Meditation>, String> {
        Ok(self
            .0
            .meditations
            .lock()
            .unwrap()
            .iter()
            .filter(|meditation| meditation.storage_key().starts_with(prefix))
            .cloned()
            .collect())
    }

    async fn set_name(&self, id: i64, name: &str) -> Result<bool, String> {
        let mut rows = self.0.meditations.lock().unwrap();
        let Some(row) = rows.iter_mut().find(|meditation| meditation.id() == id) else {
            return Ok(false);
        };
        *row = row
            .clone()
            .with_name(name)
            .map_err(|error| error.to_string())?;
        Ok(true)
    }

    async fn set_description(&self, id: i64, description: Option<&str>) -> Result<bool, String> {
        let mut rows = self.0.meditations.lock().unwrap();
        let Some(row) = rows.iter_mut().find(|meditation| meditation.id() == id) else {
            return Ok(false);
        };
        *row = row
            .clone()
            .with_description(description)
            .map_err(|error| error.to_string())?;
        Ok(true)
    }

    async fn set_storage_key(&self, id: i64, key: &str) -> Result<(), String> {
        let mut rows = self.0.meditations.lock().unwrap();
        let index = rows.iter().position(|meditation| meditation.id() == id);
        let Some(index) = index else {
            return Err(format!("meditation {id} vanished mid-relocation"));
        };
        let replacement = meditation_domain::Meditation::new(
            rows[index].id(),
            rows[index].name(),
            rows[index].content_type(),
            rows[index].size_bytes(),
            key,
            rows[index].duration_ms(),
            rows[index].created_at(),
        )
        .and_then(|row| row.with_description(rows[index].description()))
        .map_err(|error| error.to_string())?;
        rows[index] = replacement;
        Ok(())
    }
}
