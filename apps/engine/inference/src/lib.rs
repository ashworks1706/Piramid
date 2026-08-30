#![deny(missing_docs)]

//! Model execution.
//!
//! Depends on `piramid-gpu` for the device runtime and nothing in the retrieval stack; see
//! [`augment::RetrievalHook`]. Every module is a boundary with no implementation behind it yet,
//! except [`augment::RetrievalHook`], the seam everything else calls.

pub mod augment;
pub mod backends;
pub mod batching;
pub mod forward;
pub mod kv_cache;
pub mod model;
pub mod sampling;
