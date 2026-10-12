//! Startup configuration read from the environment.
//!
//! Fail-fast: a missing `MEDIOSS_DATA_DIR` aborts startup rather than silently
//! writing runtime state into the source tree (docs/ARCHITECTURE.md L359-369).

use std::{path::PathBuf, time::Duration};

use url::{Position, Url};

// The file name mirrors the domain topic; splitting it out keeps one type per file.
#[allow(clippy::module_inception)]
pub mod config;
pub mod error;
pub mod session_secret;

pub use config::Config;
pub use error::ConfigError;
pub use session_secret::SessionSecret;

const DATA_DIR_VAR: &str = "MEDIOSS_DATA_DIR";
const PORT_VAR: &str = "PORT";
const STATIC_DIR_VAR: &str = "MEDIOSS_STATIC_DIR";
const SESSION_SECRET_VAR: &str = "SESSION_SECRET";
const SESSION_TIMEOUT_VAR: &str = "SESSION_TIMEOUT";
const MEDIOSS_ORIGIN_VAR: &str = "MEDIOSS_ORIGIN";
const DEV_ORIGIN_VAR: &str = "MEDIOSS_DEV_ORIGIN";
const SESSION_COOKIE_SECURE_VAR: &str = "SESSION_COOKIE_SECURE";

const DEFAULT_PORT: u16 = 3000;
// Source-relative so `cargo run` from the repo root works without extra setup;
// the container ships its assets elsewhere and overrides MEDIOSS_STATIC_DIR.
const DEFAULT_STATIC_DIR: &str = "src/apps/web-api/static";
const DEFAULT_SESSION_TIMEOUT: &str = "7d";

/// Reads configuration from the process environment.
pub fn from_env() -> Result<Config, ConfigError> {
    let config = from_vars(&|key| std::env::var(key).ok())?;
    if !config.session_cookie_secure {
        tracing::warn!(
            "SESSION_COOKIE_SECURE=false: session cookies may be sent over unencrypted HTTP"
        );
    }
    // Module routers read this variable per request and cannot depend on
    // web-api, so publish only the validated, canonical value for them.
    match &config.dev_origin {
        Some(dev_origin) => {
            tracing::warn!(
                "{DEV_ORIGIN_VAR}={dev_origin}: this extra origin may make state-changing requests; use for development only"
            );
            std::env::set_var(DEV_ORIGIN_VAR, dev_origin);
        }
        None => std::env::remove_var(DEV_ORIGIN_VAR),
    }
    Ok(config)
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

    let session_secret = vars(SESSION_SECRET_VAR)
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .map(SessionSecret::new)
        .ok_or(ConfigError::MissingSessionSecret)?;

    let session_timeout_value =
        vars(SESSION_TIMEOUT_VAR).unwrap_or_else(|| DEFAULT_SESSION_TIMEOUT.to_owned());
    let session_timeout = parse_duration(&session_timeout_value)?;

    let medioss_origin_value = vars(MEDIOSS_ORIGIN_VAR)
        .filter(|value| !value.trim().is_empty())
        .ok_or(ConfigError::MissingMediossOrigin)?;
    let medioss_origin = parse_origin(&medioss_origin_value)?;

    let dev_origin = vars(DEV_ORIGIN_VAR)
        .filter(|value| !value.trim().is_empty())
        .map(|value| parse_origin(&value).map_err(|_| ConfigError::InvalidDevOrigin { value }))
        .transpose()?;

    let session_cookie_secure = match vars(SESSION_COOKIE_SECURE_VAR) {
        None => true,
        Some(value) if value == "true" => true,
        Some(value) if value == "false" => false,
        Some(value) => return Err(ConfigError::InvalidSessionCookieSecure { value }),
    };

    Ok(Config {
        data_dir: PathBuf::from(data_dir),
        port,
        static_dir: PathBuf::from(static_dir),
        session_secret,
        session_timeout,
        medioss_origin,
        dev_origin,
        session_cookie_secure,
    })
}

fn parse_duration(value: &str) -> Result<Duration, ConfigError> {
    let (number, unit) = value
        .split_at_checked(value.len().saturating_sub(1))
        .filter(|(number, unit)| !number.is_empty() && unit.len() == 1)
        .ok_or_else(|| invalid_session_timeout(value))?;
    let amount = number
        .parse::<u64>()
        .ok()
        .filter(|amount| *amount > 0)
        .ok_or_else(|| invalid_session_timeout(value))?;
    let seconds_per_unit = match unit {
        "d" => 24_u64 * 60 * 60,
        "h" => 60_u64 * 60,
        "m" => 60_u64,
        "s" => 1_u64,
        _ => return Err(invalid_session_timeout(value)),
    };
    let seconds = amount
        .checked_mul(seconds_per_unit)
        .ok_or_else(|| invalid_session_timeout(value))?;

    Ok(Duration::from_secs(seconds))
}

fn invalid_session_timeout(value: &str) -> ConfigError {
    ConfigError::InvalidSessionTimeout {
        value: value.to_owned(),
    }
}

fn parse_origin(value: &str) -> Result<String, ConfigError> {
    let invalid = || ConfigError::InvalidMediossOrigin {
        value: value.to_owned(),
    };
    if value != value.trim() {
        return Err(invalid());
    }
    let origin = Url::parse(value).map_err(|_| invalid())?;
    let authority_and_suffix = value
        .split_once("://")
        .map(|(_, authority_and_suffix)| authority_and_suffix)
        .ok_or_else(invalid)?;
    let suffix_start = authority_and_suffix
        .find(['/', '?', '#'])
        .unwrap_or(authority_and_suffix.len());
    let authority = &authority_and_suffix[..suffix_start];
    let suffix = &authority_and_suffix[suffix_start..];
    let is_http = matches!(origin.scheme(), "http" | "https");
    let is_origin = origin.host().is_some()
        && origin.username().is_empty()
        && origin.password().is_none()
        && !authority.contains('@')
        && matches!(suffix, "" | "/")
        && origin.path() == "/"
        && origin.query().is_none()
        && origin.fragment().is_none()
        && !value.contains('*');
    if !is_http || !is_origin {
        return Err(invalid());
    }

    Ok(origin[..Position::BeforePath].to_owned())
}

#[cfg(test)]
#[path = "config.tests.rs"]
mod tests;
