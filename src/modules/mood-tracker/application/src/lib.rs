//! Mood Tracker application commands, queries, and ports.
pub mod commands;
pub mod ports;
pub mod queries;

mod errors;

pub use errors::ApplicationError;

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

#[cfg(test)]
mod application_tests;
