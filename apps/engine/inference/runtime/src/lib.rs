//! Model execution.
//!
//! # Layout
//!
//! - [`model`] — architecture definitions and weight loading.
//! - [`runtime`] — the forward-pass driver.
//! - [`kv_cache`] — attention key/value cache ownership.
//! - [`batching`] — request admission and batch assembly.
//! - [`sampling`] — logits to tokens.
//! - [`fusion`] — where retrieval enters the forward pass (re-exported from `piramid-fusion`).
//! - [`backends`] — execution backends (candle today).
//!
//! # Layering
//!
//! This crate depends on `piramid-gpu` for the device runtime and on `piramid-fusion` for the
//! hook trait — and on **nothing in the retrieval stack**. A concrete fusion strategy is a
//! separate crate that depends on both `piramid-fusion` and `piramid-search`, which keeps the
//! model runtime independent of retrieval and leaves a collection queryable with no model loaded.
//!
//! Backend crates stay confined to [`backends`], the same containment rule
//! `piramid-gpu::backends` follows.
//!
//! # Status
//!
//! Skeleton. Every module here is a boundary with its contract written down and no implementation
//! behind it. The one piece that matters to get right early is [`fusion::RetrievalHook`] — see
//! that module for why it is defined before anything that could call it.

pub mod backends;
pub mod batching;
pub mod kv_cache;
pub mod model;
pub mod runtime;
pub mod sampling;

// The fusion seam is its own crate so a strategy can depend on retrieval without this one doing
// so. Re-exported for callers, who should not have to know where the boundary falls.
pub use piramid_fusion as fusion;
