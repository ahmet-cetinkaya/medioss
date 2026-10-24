/// Stores an uploaded icon image (sniffed png/jpeg/webp, <= 1 MB), replacing
/// any previous one.
#[derive(Debug, Clone)]
pub struct SetSoundIconCommand {
    pub id: i64,
    pub bytes: Vec<u8>,
}
