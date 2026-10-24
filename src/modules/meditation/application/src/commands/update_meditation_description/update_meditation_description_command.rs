#[derive(Debug, Clone)]
pub struct UpdateMeditationDescriptionCommand {
    pub meditation_id: i64,
    pub description: Option<String>,
}
