#![deny(missing_docs)]

//! Model execution.
//!
//! Modules: [`model`] (architecture, weight loading), [`forward`] (the driver loop),
//! [`kv_cache`], [`batching`], [`sampling`] (logits to tokens), [`augment`] (the retrieval seam),
//! [`backends`] (execution backends, candle today).
//!
//! Depends on `piramid-gpu` for the device runtime and on nothing in the retrieval stack:
//! [`augment::RetrievalHook`] is only a trait, a strategy that actually queries an index is a
//! separate crate depending on this one and `piramid-search`, enforced by
//! `scripts/check-deps.sh`. Backend crates stay confined to [`backends`], the same rule
//! `piramid-gpu::backends` follows.
//!
//! Skeleton: every module is a boundary with no implementation behind it yet, except
//! [`augment::RetrievalHook`], which exists first because it is the seam everything else calls.

pub mod augment;
pub mod backends;
pub mod batching;
pub mod forward;
pub mod kv_cache;
pub mod model;
pub mod sampling;
