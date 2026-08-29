//! Model execution.
//!
//! # Layout
//!
//! - [`model`] — architecture definitions and weight loading.
//! - [`runtime`] — the forward-pass driver.
//! - [`kv_cache`] — attention key/value cache ownership.
//! - [`batching`] — request admission and batch assembly.
//! - [`sampling`] — logits to tokens.
//! - [`fusion`] — where retrieval enters the forward pass.
//! - [`backends`] — execution backends (candle today).
//!
//! # Layering
//!
//! Inference depends on `piramid-gpu` for the device runtime and on `piramid-search` for
//! retrieval, but retrieval must never depend on inference: a collection has to remain queryable
//! with no model loaded. Backend crates stay confined to [`backends`], the same containment rule
//! `piramid-gpu::backends` follows.
//!
//! # Status
//!
//! Skeleton. Every module here is a boundary with its contract written down and no implementation
//! behind it. The one piece that matters to get right early is [`fusion::RetrievalHook`] — see
//! that module for why it is defined before anything that could call it.

pub mod backends;
pub mod batching;
pub mod fusion;
pub mod kv_cache;
pub mod model;
pub mod runtime;
pub mod sampling;
