//! Router-level tests: authorization gates, streaming Range matrix, and
//! error mapping against fake ports (identity router_tests style).

mod access;
mod audio;
mod covers;
mod description;
mod favorite_ids_unavailable;
mod helpers;
mod library;
mod listening;
mod reorder;
mod statistics;

pub(crate) use helpers::*;
