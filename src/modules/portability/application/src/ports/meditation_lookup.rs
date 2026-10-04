use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct MeditationRef {
    pub id: i64,
    pub name: String,
    pub duration_ms: i64,
}

#[async_trait]
pub trait MeditationLookup: Send + Sync {
    async fn list_all(&self) -> Result<Vec<MeditationRef>, String>;
}
