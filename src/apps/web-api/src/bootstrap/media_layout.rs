//! One-time migration of `media/` to the module-prefixed layout.
//!
//! Releases before 0.5.0 stored every module's blobs directly under
//! `media/`: program folders and `_unfiled/` (meditation audio and covers),
//! `soundscape/`, and numeric `{user_id}/` avatar folders. The prefixed
//! layout gives each module its own root — `media/meditation/`,
//! `media/soundscape/`, `media/identity/` — so a per-module backup or wipe
//! never needs a path list. The storage keys in the database are relative
//! to each module's root and unchanged; only the on-disk location moves.
//!
//! Runs once at startup, before any store is wired: every legacy top-level
//! child of `media/` is renamed into its module root. Idempotent — a second
//! run finds nothing to move — and it only renames, never deletes, so an
//! interrupted run resumes safely and unrecognized children are left alone.

use std::path::Path;

/// Moves legacy `media/` children into their module roots.
pub fn migrate_media_layout(data_dir: &Path) {
    let media = data_dir.join("media");
    let Ok(entries) = std::fs::read_dir(&media) else {
        return; // No media directory yet: nothing to migrate.
    };
    for entry in entries.flatten() {
        let Some((_module, destination)) = migration_target(&media, &entry.file_name()) else {
            continue;
        };
        let from = entry.path();
        if let Err(err) = std::fs::create_dir_all(destination.parent().unwrap()) {
            tracing::warn!(
                "media migration could not create {}: {err}",
                destination.parent().unwrap().display()
            );
            continue;
        }
        if destination.exists() {
            tracing::warn!(
                "media migration skipped {}: {} already exists",
                from.display(),
                destination.display()
            );
            continue;
        }
        match std::fs::rename(&from, &destination) {
            Ok(()) => {
                tracing::info!(
                    "media migration: {} -> {}",
                    from.display(),
                    destination.display()
                )
            }
            // A rename can fail across filesystems; fall back to copy+delete.
            Err(_) => {
                if let Err(err) =
                    std::fs::copy(&from, &destination).and_then(|_| std::fs::remove_file(&from))
                {
                    tracing::warn!("media migration failed for {}: {err}", from.display());
                }
            }
        }
    }
}

/// Where a legacy `media/` child moves: `(module name, target path)`.
/// Returns `None` for anything already module-prefixed or unrecognized —
/// an unknown child is never moved, only a human can decide about it.
fn migration_target(
    media: &Path,
    name: &std::ffi::OsStr,
) -> Option<(&'static str, std::path::PathBuf)> {
    let name = name.to_str()?;
    if name == "soundscape" || name == "meditation" || name == "identity" || name.contains('.') {
        return None; // Already module-prefixed roots; stray files stay put.
    }
    if is_user_id(name) {
        return Some(("identity", media.join("identity").join(name)));
    }
    if is_program_folder(name) {
        return Some(("meditation", media.join("meditation").join(name)));
    }
    None
}

/// User media folders are named by the numeric `UserId`.
fn is_user_id(name: &str) -> bool {
    name.parse::<u64>().is_ok()
}

/// Program folders are slugs (`Mindful-Walks-3a1b`) or the `_unfiled/`
/// holding area; both are filename-safe with no dots.
fn is_program_folder(name: &str) -> bool {
    name == "_unfiled"
        || name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[cfg(test)]
#[path = "media_layout.tests.rs"]
mod tests;
