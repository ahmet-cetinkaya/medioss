//! Persistence: repositories implementing the application ports over
//! SeaORM/SQLite. One file per repository (identity convention).

pub mod category_repository;
pub mod clock;
pub mod entities;
pub mod favorite_repository;
pub mod meditation_repository;
pub mod membership_repository;
pub mod note_repository;
pub mod playback_repository;
pub mod program_repository;
pub mod session_repository;

pub use category_repository::SeaOrmCategoryRepository;
pub use clock::SystemClock;
pub use favorite_repository::SeaOrmFavoriteRepository;
pub use meditation_repository::SeaOrmMeditationRepository;
pub use membership_repository::SeaOrmMembershipRepository;
pub use note_repository::SeaOrmNoteRepository;
pub use playback_repository::SeaOrmPlaybackRepository;
pub use program_repository::SeaOrmProgramRepository;
pub use session_repository::SeaOrmSessionRepository;

#[cfg(test)]
#[path = "persistence.tests.rs"]
mod repository_tests;

/// Slugifies a program title for its storage folder.
pub(crate) fn folder_for(title: &str, suffix: String) -> String {
    let folded: String = title
        .chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| match c {
            'ç' => 'c',
            'ğ' => 'g',
            'ı' => 'i',
            'ö' => 'o',
            'ş' => 's',
            'ü' => 'u',
            other if other.is_ascii_alphanumeric() => other,
            _ => '-',
        })
        .collect();
    let mut slug = String::new();
    let mut previous_dash = false;
    for c in folded.chars() {
        if c == '-' {
            if !previous_dash && !slug.is_empty() {
                slug.push('-');
                previous_dash = true;
            }
        } else {
            slug.push(c);
            previous_dash = false;
        }
        if slug.chars().count() >= 40 {
            break;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        slug = "program".to_owned();
    }
    format!("{slug}-{suffix}")
}

/// Per-insert unique suffix (time+counter mix; O_EXCL keeps collisions rare).
pub(crate) fn model_suffix() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mix = nanos ^ (count.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    format!("{:06x}", (mix as u32 as u64 ^ (mix >> 32)) & 0xFF_FFFF)
}
