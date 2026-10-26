use std::sync::Arc;

use crate::{
    bootstrap::{database::Database, report::fail},
    config::Config,
    state::AppState,
};

/// Assembles the shared application state; a session-store migration failure exits.
pub async fn build_state(config: Config, database: Database) -> AppState {
    let session_store = tower_sessions_sqlx_store::SqliteStore::new(database.pool.clone());
    if let Err(err) = session_store.migrate().await {
        fail(format!("session store migration failed: {err}"));
    }
    let hasher = identity_infrastructure::BoundedArgon2PasswordHasher::new();
    let origin = || Arc::new(config.medioss_origin.clone());
    let meditation_module = crate::meditation::meditation_dependencies(
        database.connection.clone(),
        &config.data_dir,
        origin(),
        config.upload_limits,
    );
    let portability =
        crate::portability::portability_dependencies(database.connection.clone(), origin());
    let (mood_tracker, mood_tracker_entries) =
        crate::mood_tracker::mood_tracker_dependencies(database.connection.clone(), origin());
    let soundscape = crate::soundscape::soundscape_dependencies(
        database.connection.clone(),
        &config.data_dir,
        origin(),
        config.upload_limits.sound_audio_bytes,
    );
    let deletion: Arc<dyn identity_presentation::UserDeletionListener> =
        Arc::new(crate::user_deletion::CrossModuleUserDeletion::new(
            Arc::clone(&meditation_module.notes),
            Arc::clone(&meditation_module.playback),
            Arc::clone(&meditation_module.sessions),
            Arc::clone(&soundscape.mixes),
            Arc::clone(&mood_tracker_entries),
            config.data_dir.join("media").join("identity"),
        ));
    let avatar_store = Arc::new(crate::avatar_store::FileAvatarStore::new(
        config.data_dir.join("media").join("identity"),
    ));
    AppState::new(
        Arc::new(config),
        Arc::new(session_store),
        meditation_module.shared,
        mood_tracker,
        soundscape,
        portability,
        Arc::new(identity_infrastructure::SeaOrmUserRepository::new(
            database.connection,
        )),
        hasher,
        deletion,
        avatar_store,
    )
}

#[cfg(test)]
#[path = "app_state.tests.rs"]
mod tests;
