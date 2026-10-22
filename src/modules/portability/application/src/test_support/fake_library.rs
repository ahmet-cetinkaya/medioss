use crate::ports::meditation_lookup::{MeditationLookup, MeditationRef};
use std::sync::Mutex;

pub struct FakeLibrary(pub Mutex<Vec<MeditationRef>>);

#[async_trait::async_trait]
impl MeditationLookup for FakeLibrary {
    async fn list_all(&self) -> Result<Vec<MeditationRef>, String> {
        Ok(self.0.lock().unwrap().clone())
    }
}
