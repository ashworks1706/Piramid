//! Device kernel sources and their typed launch wrappers.
//!
//! # Convention
//!
//! Each kernel family is a pair of files that sit side by side:
//!
//! ```text
//! distance.cu    // device code
//! distance.rs    // typed launch wrapper: builds a LaunchConfig, binds args, launches
//! ```
//!
//! The `.rs` wrapper is the only thing the rest of the crate calls. It owns argument binding and
//! launch geometry; it does not own device lifetime or memory, which come from
//! [`crate::device`] and [`crate::buffer`].
//!
//! Nothing here decides *which* backend runs — that is `piramid-compute::backends`. These modules
//! are reached only after a GPU backend has already been selected.

pub mod attention;
pub mod distance;
pub mod quantize;
