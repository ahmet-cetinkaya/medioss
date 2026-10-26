//! Shared fixtures for the crate's unit tests.

use std::sync::Arc;

use crate::state::AppState;

pub(crate) async fn test_state() -> (AppState, tempfile::TempDir) {
    test_state_with_cookie_secure(false).await
}

pub(crate) async fn test_state_with_cookie_secure(
    session_cookie_secure: bool,
) -> (AppState, tempfile::TempDir) {
    let directory = tempfile::tempdir().unwrap();
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(directory.path().join("test.sqlite"))
                .create_if_missing(true),
        )
        .await
        .unwrap();
    let connection = sea_orm::SqlxSqliteConnector::from_sqlx_sqlite_pool(pool.clone());
    identity_infrastructure::migrate(&connection).await.unwrap();
    meditation_infrastructure::migrations::migrate(&connection)
        .await
        .unwrap();
    soundscape_infrastructure::migrations::migrate(&connection)
        .await
        .unwrap();
    mood_tracker_infrastructure::migrations::migrate(&connection)
        .await
        .unwrap();
    let store = tower_sessions_sqlx_store::SqliteStore::new(pool);
    store.migrate().await.unwrap();

    let config = test_config(directory.path(), session_cookie_secure);

    let origin = Arc::new(config.medioss_origin.clone());
    let meditation_module = crate::meditation::meditation_dependencies(
        connection.clone(),
        directory.path(),
        Arc::clone(&origin),
        config.upload_limits,
    );
    let soundscape = crate::soundscape::soundscape_dependencies(
        connection.clone(),
        directory.path(),
        Arc::clone(&origin),
        config.upload_limits.sound_audio_bytes,
    );
    let (mood_tracker, mood_tracker_entries) =
        crate::mood_tracker::mood_tracker_dependencies(connection.clone(), Arc::clone(&origin));
    let portability =
        crate::portability::portability_dependencies(connection.clone(), Arc::clone(&origin));
    let deletion: std::sync::Arc<dyn identity_presentation::UserDeletionListener> =
        std::sync::Arc::new(crate::user_deletion::CrossModuleUserDeletion::new(
            meditation_module.notes.clone(),
            meditation_module.favorites.clone(),
            meditation_module.playback.clone(),
            meditation_module.sessions.clone(),
            soundscape.mixes.clone(),
            mood_tracker_entries.clone(),
            directory.path().join("media"),
        ));
    let state = AppState::new(
        Arc::new(config),
        Arc::new(store),
        meditation_module.shared,
        mood_tracker,
        soundscape,
        portability,
        Arc::new(identity_infrastructure::SeaOrmUserRepository::new(
            connection,
        )),
        identity_infrastructure::BoundedArgon2PasswordHasher::new(),
        deletion,
        Arc::new(crate::avatar_store::FileAvatarStore::new(
            directory.path().join("media"),
        )),
    );
    (state, directory)
}

pub(crate) fn test_config(
    data_dir: &std::path::Path,
    session_cookie_secure: bool,
) -> crate::config::Config {
    crate::config::Config {
        data_dir: data_dir.to_path_buf(),
        port: 0,
        static_dir: data_dir.to_path_buf(),
        bind_host: "127.0.0.1".parse().unwrap(),
        session_secret: crate::config::SessionSecret::new_for_tests("test-secret"),
        session_timeout: std::time::Duration::from_secs(3600),
        medioss_origin: "https://medioss.example".to_owned(),
        dev_origin: None,
        session_cookie_secure,
        upload_limits: crate::config::UploadLimits::default(),
    }
}
