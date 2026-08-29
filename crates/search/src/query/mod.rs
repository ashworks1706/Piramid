//! Query types.
//!
//! [`Filter`] and [`FilterCondition`] are defined in `metadata/` and re-exported here so the
//! search API keeps a single import path for everything a caller composes a query from.

pub use piramid_core::metadata::{Filter, FilterCondition};
