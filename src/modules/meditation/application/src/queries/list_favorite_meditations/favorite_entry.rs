use meditation_domain::Meditation;

use super::favorite_program::FavoriteProgram;

/// A favorite meditation with the programs it belongs to (earliest
/// membership first), so the list can name them without more requests.
#[derive(Debug, Clone)]
pub struct FavoriteEntry {
    pub meditation: Meditation,
    pub programs: Vec<FavoriteProgram>,
}
