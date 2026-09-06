//! Shared handler state.
//!
//! M1 will add services here (db pool, media-store instance, session service).

use std::sync::Arc;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
}

// M0: proves workspace crates link into the binary (removed in M1 when real wiring lands)
#[allow(dead_code)]
pub type KernelWiring = kernel::Actor;
#[allow(dead_code)]
pub type MediaStoreWiring = media_store::MediaStore;
