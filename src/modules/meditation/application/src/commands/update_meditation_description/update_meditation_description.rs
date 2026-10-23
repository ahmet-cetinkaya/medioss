#[derive(Debug, Clone)]
pub struct UpdateMeditationDescription {
    pub meditation_id: i64,
    pub description: Option<String>,
}
