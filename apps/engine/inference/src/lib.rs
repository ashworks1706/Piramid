//! Model execution.
//!
//! # Layout
//!
//! - [`model`] — architecture definitions and weight loading.
//! - [`forward`] — the forward-pass driver.
//! - [`kv_cache`] — attention key/value cache ownership.
//! - [`batching`] — request admission and batch assembly.
//! - [`sampling`] — logits to tokens.
//! - [`augment`] — the seam where retrieval enters the forward pass.
//! - [`backends`] — execution backends (candle today).
//!
//! # Layering
//!
//! This crate depends on `piramid-gpu` for the device runtime and on **nothing in the retrieval
//! stack**. [`augment::RetrievalHook`] is only the trait; a strategy that actually queries an
//! index is a separate crate depending on both this one and `piramid-search`. That keeps the
//! model runtime independent of retrieval and leaves a collection queryable with no model loaded,
//! enforced by `scripts/check-deps.sh`.
//!
//! Backend crates stay confined to [`backends`], the same containment rule
//! `piramid-gpu::backends` follows.
//!
//! # Status
//!
//! Skeleton. Every module here is a boundary with its contract written down and no implementation
//! behind it. The one piece that matters to get right early is [`augment::RetrievalHook`] — see
//! that module for why it is defined before anything that could call it.

pub mod augment;
pub mod backends;
pub mod batching;
pub mod forward;
pub mod kv_cache;
pub mod model;
pub mod sampling;
