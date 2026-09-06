//! Shared handler state.
//!
//! M1 will add services here (db pool, media-store instance, session service).

use std::sync::Arc;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
}

// M0: proves workspace crates link into the binary without exposing them
// (removed in M1 when real wiring lands and these become genuine fields).
use kernel as _;
use media_store as _;
