//! Query types.
//!
//! [`Filter`] and [`FilterCondition`] are defined in `metadata/` and re-exported here, so a
//! caller composes a query from one import path.

pub use piramid_core::metadata::{Filter, FilterCondition};
