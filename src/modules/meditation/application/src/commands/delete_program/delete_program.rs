#[derive(Debug, Clone)]
pub struct DeleteProgram {
    pub program_id: i64,
}

/// Shared folder that receives meditations when their program is deleted.
pub const UNFILED: &str = "_unfiled";
