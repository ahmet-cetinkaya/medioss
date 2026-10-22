// The layout convention (docs/ARCHITECTURE.md) puts each use case in a folder
// named after it, with the command struct in <name>/<name>.rs — that nesting
// is deliberate, not accidental inception.
#![allow(clippy::module_inception)]

pub mod commands;
pub mod models;
pub mod ports;
pub mod queries;

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

/// The export document format this workspace writes and reads. Bump only with
/// a migration path for readers of the previous version.
pub const FORMAT_VERSION: u32 = 1;
