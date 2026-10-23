//! Meditation application: commands, queries, ports, and shared services.

// The layout convention (docs/ARCHITECTURE.md) puts each use case in a folder
// named after it, with the command struct in <name>/<name>.rs — that nesting
// is deliberate, not accidental inception.
#![allow(clippy::module_inception)]

pub mod commands;
pub mod models;
pub mod ports;
pub mod queries;
pub mod services;

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

pub use models::errors::ApplicationError;
