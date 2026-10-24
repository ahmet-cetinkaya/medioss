use std::sync::Mutex;

use async_trait::async_trait;
use kernel::UserId;
use soundscape_domain::Mix;

use crate::ports::{MixRepository, NewMix};

#[derive(Default)]
pub struct InMemoryMixes {
    pub mixes: Mutex<Vec<Mix>>,
    /// When set, every operation fails with this message.
    pub failure: Option<String>,
}

impl InMemoryMixes {
    pub fn failing(message: &str) -> Self {
        Self {
            failure: Some(message.into()),
            ..Self::default()
        }
    }

    pub fn snapshot(&self) -> Vec<Mix> {
        self.mixes.lock().unwrap().clone()
    }

    fn check(&self) -> Result<(), String> {
        self.failure.clone().map_or(Ok(()), Err)
    }

    fn build(id: i64, mix: NewMix) -> Result<Mix, String> {
        Mix::new(
            id,
            mix.user_id.value(),
            &mix.name,
            mix.items,
            mix.now,
            mix.now,
        )
        .map_err(|error| error.to_string())
    }
}

#[async_trait]
impl MixRepository for InMemoryMixes {
    async fn list_for(&self, user_id: UserId) -> Result<Vec<Mix>, String> {
        self.check()?;
        Ok(self
            .snapshot()
            .into_iter()
            .filter(|mix| mix.user_id == user_id.value())
            .collect())
    }

    async fn find_for(&self, id: i64, user_id: UserId) -> Result<Option<Mix>, String> {
        self.check()?;
        Ok(self
            .snapshot()
            .into_iter()
            .find(|mix| mix.id == id && mix.user_id == user_id.value()))
    }

    async fn create(&self, mix: NewMix) -> Result<Mix, String> {
        self.check()?;
        let mut mixes = self.mixes.lock().unwrap();
        let created = Self::build(mixes.len() as i64 + 1, mix)?;
        mixes.push(created.clone());
        Ok(created)
    }

    async fn replace(&self, id: i64, mix: NewMix) -> Result<Option<Mix>, String> {
        self.check()?;
        let mut mixes = self.mixes.lock().unwrap();
        let Some(slot) = mixes
            .iter_mut()
            .find(|existing| existing.id == id && existing.user_id == mix.user_id.value())
        else {
            return Ok(None);
        };
        *slot = Self::build(id, mix)?;
        Ok(Some(slot.clone()))
    }

    async fn delete_for(&self, id: i64, user_id: UserId) -> Result<bool, String> {
        self.check()?;
        let mut mixes = self.mixes.lock().unwrap();
        let before = mixes.len();
        mixes.retain(|mix| !(mix.id == id && mix.user_id == user_id.value()));
        Ok(mixes.len() < before)
    }

    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, String> {
        self.check()?;
        let mut mixes = self.mixes.lock().unwrap();
        let before = mixes.len();
        mixes.retain(|mix| mix.user_id != user_id.value());
        Ok((before - mixes.len()) as u64)
    }
}
