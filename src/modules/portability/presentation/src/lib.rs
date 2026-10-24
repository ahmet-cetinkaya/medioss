// Presentation is organized by file type: handlers/ (one per endpoint),
// formats/ (serialization), support/ (cross-cutting helpers). The
// folder-per-file nesting below is deliberate, not module inception.
#![allow(clippy::module_inception)]

pub mod extractors;
pub mod formats;
pub mod handlers;
pub mod router;
pub mod support;

#[cfg(test)]
mod test_support;

pub use router::SharedDependencies;
