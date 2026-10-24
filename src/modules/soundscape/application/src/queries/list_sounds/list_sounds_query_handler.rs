use std::sync::Arc;

use super::list_sounds_query::ListSoundsQuery;
use super::list_sounds_query_validator::ListSoundsQueryValidator;
use crate::ports::{QueryHandler, SoundRepository};
use crate::ApplicationError;
use soundscape_domain::Sound;

pub struct ListSoundsQueryHandler {
    sounds: Arc<dyn SoundRepository>,
    validator: ListSoundsQueryValidator,
}

impl ListSoundsQueryHandler {
    pub fn new(sounds: Arc<dyn SoundRepository>) -> Self {
        Self {
            sounds,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<ListSoundsQuery> for ListSoundsQueryHandler {
    type Output = Vec<Sound>;

    async fn handle(&self, query: ListSoundsQuery) -> Result<Vec<Sound>, ApplicationError> {
        self.validator.validate(&query)?;
        self.sounds
            .list()
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "list_sounds_query_handler.tests.rs"]
mod tests;
