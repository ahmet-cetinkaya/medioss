//! The resolved startup configuration.
//!
//! Re-exported as `crate::config::Config`; construct it via `config::from_env`
//! or `config::from_vars`.

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub data_dir: PathBuf,
    pub port: u16,
    pub static_dir: PathBuf,
}
