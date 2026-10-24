use crate::{bootstrap::database::Database, config::Config};

/// Seeds the built-in soundscape set when present; a failed seed only warns.
pub async fn seed_builtin_soundscapes(database: &Database, config: &Config) {
    match soundscape_infrastructure::builtin::seed_if_present(
        &database.connection,
        &config.data_dir.join("media"),
    )
    .await
    {
        Ok(true) => tracing::info!("built-in soundscape set seeded"),
        Ok(false) => tracing::debug!("no built-in soundscape set found; skipping seed"),
        Err(err) => tracing::warn!("built-in soundscape seed failed: {err}"),
    }
}

#[cfg(test)]
#[path = "builtin_sounds.tests.rs"]
mod tests;
