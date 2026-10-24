use std::sync::Mutex;

use async_trait::async_trait;
use soundscape_domain::SoundCategory;

use crate::ports::{CategoryDelete, CategoryRepository};

#[derive(Default)]
pub struct InMemoryCategories {
    pub categories: Mutex<Vec<SoundCategory>>,
    pub in_use_ids: Vec<i64>,
    /// When set, every operation fails with this message.
    pub failure: Option<String>,
}

impl InMemoryCategories {
    pub fn with(categories: Vec<SoundCategory>) -> Self {
        Self {
            categories: Mutex::new(categories),
            ..Self::default()
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            failure: Some(message.into()),
            ..Self::default()
        }
    }

    fn check(&self) -> Result<(), String> {
        self.failure.clone().map_or(Ok(()), Err)
    }
}

#[async_trait]
impl CategoryRepository for InMemoryCategories {
    async fn list(&self) -> Result<Vec<SoundCategory>, String> {
        self.check()?;
        Ok(self.categories.lock().unwrap().clone())
    }

    async fn find(&self, id: i64) -> Result<Option<SoundCategory>, String> {
        self.check()?;
        Ok(self
            .categories
            .lock()
            .unwrap()
            .iter()
            .find(|category| category.id == id)
            .cloned())
    }

    async fn create(&self, name: &str) -> Result<SoundCategory, String> {
        self.check()?;
        let mut categories = self.categories.lock().unwrap();
        let category = SoundCategory::new(categories.len() as i64 + 1, name, name)
            .map_err(|error| error.to_string())?;
        categories.push(category.clone());
        Ok(category)
    }

    async fn rename(&self, id: i64, name: &str) -> Result<Option<SoundCategory>, String> {
        self.check()?;
        let mut categories = self.categories.lock().unwrap();
        let Some(category) = categories.iter_mut().find(|category| category.id == id) else {
            return Ok(None);
        };
        category.name = name.into();
        Ok(Some(category.clone()))
    }

    async fn delete_empty(&self, id: i64) -> Result<CategoryDelete, String> {
        self.check()?;
        if self.in_use_ids.contains(&id) {
            return Ok(CategoryDelete::InUse);
        }
        let mut categories = self.categories.lock().unwrap();
        let before = categories.len();
        categories.retain(|category| category.id != id);
        Ok(if categories.len() < before {
            CategoryDelete::Deleted
        } else {
            CategoryDelete::NotFound
        })
    }
}
