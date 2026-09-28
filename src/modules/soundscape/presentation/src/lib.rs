pub mod router;
pub use router::{soundscape_router, SharedDependencies, SoundscapeDependencies};

#[cfg(test)]
#[path = "router.tests.rs"]
mod router_tests;
