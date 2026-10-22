use super::meditation_ref::MeditationRef;

#[async_trait::async_trait]
pub trait MeditationLookup: Send + Sync {
    async fn list_all(&self) -> Result<Vec<MeditationRef>, String>;
}
