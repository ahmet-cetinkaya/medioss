//! Startup configuration read from the environment.
//!
//! Fail-fast: a missing `MEDIOSS_DATA_DIR` aborts startup rather than silently
//! writing runtime state into the source tree (docs/ARCHITECTURE.md L359-369).

use std::path::PathBuf;

// The file name mirrors the domain topic; splitting it out keeps one type per file.
#[allow(clippy::module_inception)]
pub mod config;
pub mod error;

pub use config::Config;
pub use error::ConfigError;

const DATA_DIR_VAR: &str = "MEDIOSS_DATA_DIR";
const PORT_VAR: &str = "PORT";
const STATIC_DIR_VAR: &str = "MEDIOSS_STATIC_DIR";

const DEFAULT_PORT: u16 = 3000;
// Source-relative so `cargo run` from the repo root works without extra setup;
// the container ships its assets elsewhere and overrides MEDIOSS_STATIC_DIR.
const DEFAULT_STATIC_DIR: &str = "src/apps/web-api/static";

/// Reads configuration from the process environment.
pub fn from_env() -> Result<Config, ConfigError> {
    from_vars(&|key| std::env::var(key).ok())
}

/// Pure configuration parsing: no filesystem access, no directory creation.
pub fn from_vars(vars: &dyn Fn(&str) -> Option<String>) -> Result<Config, ConfigError> {
    let data_dir = vars(DATA_DIR_VAR)
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .ok_or(ConfigError::MissingDataDir)?;

    let port = match vars(PORT_VAR) {
        Some(value) => value
            .parse()
            .map_err(|_| ConfigError::InvalidPort { value })?,
        None => DEFAULT_PORT,
    };

    let static_dir = vars(STATIC_DIR_VAR).unwrap_or_else(|| DEFAULT_STATIC_DIR.to_owned());

    Ok(Config {
        data_dir: PathBuf::from(data_dir),
        port,
        static_dir: PathBuf::from(static_dir),
    })
}

#[cfg(test)]
#[path = "config.tests.rs"]
mod tests;
