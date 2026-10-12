//! The resolved startup configuration.
//!
//! Re-exported as `crate::config::Config`; construct it via `config::from_env`
//! or `config::from_vars`.

use std::{path::PathBuf, time::Duration};

use super::SessionSecret;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub data_dir: PathBuf,
    pub port: u16,
    pub static_dir: PathBuf,
    pub session_secret: SessionSecret,
    pub session_timeout: Duration,
    pub medioss_origin: String,
    pub dev_origin: Option<String>,
    pub session_cookie_secure: bool,
}
