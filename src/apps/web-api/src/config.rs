//! Startup configuration read from the environment.
//!
//! Fail-fast: a missing `MEDIOSS_DATA_DIR` aborts startup rather than silently
//! writing runtime state into the source tree (docs/ARCHITECTURE.md L359-369).

use std::path::PathBuf;

const DATA_DIR_VAR: &str = "MEDIOSS_DATA_DIR";
const PORT_VAR: &str = "PORT";
const STATIC_DIR_VAR: &str = "MEDIOSS_STATIC_DIR";

const DEFAULT_PORT: u16 = 3000;
const DEFAULT_STATIC_DIR: &str = "src/apps/web-api/static";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub data_dir: PathBuf,
    pub port: u16,
    pub static_dir: PathBuf,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error(
        "MEDIOSS_DATA_DIR is required: set it to the writable directory that will hold medioss.db and media/"
    )]
    MissingDataDir,

    #[error("PORT must be a number between 0 and 65535, got `{value}`")]
    InvalidPort { value: String },
}

/// Reads configuration from the process environment.
pub fn from_env() -> Result<Config, ConfigError> {
    from_vars(&|key| std::env::var(key).ok())
}

/// Pure configuration parsing: no filesystem access, no directory creation.
pub fn from_vars(vars: &dyn Fn(&str) -> Option<String>) -> Result<Config, ConfigError> {
    let data_dir = vars(DATA_DIR_VAR)
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
mod tests {
    use super::*;

    const MISSING_DATA_DIR_MESSAGE: &str = "MEDIOSS_DATA_DIR is required: set it to the writable directory that will hold medioss.db and media/";

    fn vars(pairs: &[(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
        let pairs = pairs.to_vec();
        move |key| {
            pairs
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).to_owned())
        }
    }

    #[test]
    fn missing_data_dir_is_an_error_with_the_documented_message() {
        let error = from_vars(&|_| None).unwrap_err();

        assert_eq!(error, ConfigError::MissingDataDir);
        assert_eq!(error.to_string(), MISSING_DATA_DIR_MESSAGE);
    }

    #[test]
    fn empty_data_dir_is_an_error_with_the_documented_message() {
        let error = from_vars(&vars(&[("MEDIOSS_DATA_DIR", "")])).unwrap_err();

        assert_eq!(error, ConfigError::MissingDataDir);
        assert_eq!(error.to_string(), MISSING_DATA_DIR_MESSAGE);
    }

    #[test]
    fn port_defaults_to_3000() {
        let config = from_vars(&vars(&[("MEDIOSS_DATA_DIR", "/data")])).unwrap();

        assert_eq!(config.port, 3000);
        assert_eq!(config.data_dir, PathBuf::from("/data"));
    }

    #[test]
    fn port_override_is_respected() {
        let config = from_vars(&vars(&[("MEDIOSS_DATA_DIR", "/data"), ("PORT", "8080")])).unwrap();

        assert_eq!(config.port, 8080);
    }

    #[test]
    fn invalid_port_names_the_variable() {
        let error =
            from_vars(&vars(&[("MEDIOSS_DATA_DIR", "/data"), ("PORT", "http")])).unwrap_err();

        assert!(error.to_string().contains("PORT"));
    }

    #[test]
    fn static_dir_defaults_into_the_source_tree_and_is_overridable() {
        let default = from_vars(&vars(&[("MEDIOSS_DATA_DIR", "/data")])).unwrap();
        assert_eq!(default.static_dir, PathBuf::from("src/apps/web-api/static"));

        let overridden = from_vars(&vars(&[
            ("MEDIOSS_DATA_DIR", "/data"),
            ("MEDIOSS_STATIC_DIR", "/srv/static"),
        ]))
        .unwrap();
        assert_eq!(overridden.static_dir, PathBuf::from("/srv/static"));
    }
}
