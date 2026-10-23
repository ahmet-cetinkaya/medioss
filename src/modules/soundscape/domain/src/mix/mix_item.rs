use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MixItem {
    pub sound_id: i64,
    pub volume: i64,
    pub position: i64,
}
