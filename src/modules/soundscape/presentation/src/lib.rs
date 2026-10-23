// The layout convention (docs/ARCHITECTURE.md) organizes this crate by role
// (handlers/, support/, extractors, router); same-name file nesting is
// deliberate, not module inception.
#![allow(clippy::module_inception)]

mod extractors;
mod handlers;
mod router;
mod support;

#[cfg(test)]
#[path = "router.tests.rs"]
mod router_tests;

#[cfg(test)]
mod test_support;

pub use router::{soundscape_router, SharedDependencies, SoundscapeDependencies};
