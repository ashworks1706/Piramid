//! Device kernel sources and their typed launch wrappers.
//!
//! Each kernel family is two files side by side: `distance.cu` holds the device code, `distance.rs`
//! the typed wrapper that builds a `LaunchConfig`, binds args and launches. The wrapper is the
//! only thing the rest of the crate calls; device lifetime and memory come from [`crate::device`]
//! and [`crate::buffer`].
//!
//! Nothing here decides which backend runs. These modules are reached only after
//! `piramid-compute::backends` has already selected a GPU.

pub mod attention;
pub mod distance;
pub mod quantize;
