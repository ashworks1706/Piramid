//! Retrieval fused into the forward pass.
//!
//! This crate is one trait and the types it needs. It is the reason the rest of the workspace is
//! shaped the way it is.
//!
//! Conventional RAG concatenates retrieved text into the prompt: retrieval finishes before
//! generation starts, and the evidence costs context-window tokens on every query. Fusion instead
//! lets the model consult an index *during* the forward pass, so evidence enters through hidden
//! state rather than through the token stream.
//!
//! # Why this is its own crate
//!
//! It depends on `piramid-core` and nothing else — deliberately. `piramid-inference` (the forward
//! pass driver) depends on this crate, so the driver has the hook call sites without depending on
//! retrieval. A concrete fusion strategy is a *separate* crate that depends on both this one and
//! `piramid-search`.
//!
//! ```text
//! inference/runtime ──→ inference/fusion ←── a future fusion strategy ──→ retrieval/search
//! ```
//!
//! That direction is the whole point: the model runtime never depends on the retrieval stack, so
//! a model can run with fusion disabled and a collection stays queryable with no model loaded.
//!
//! # Why the trait exists before anything calls it
//!
//! A forward-pass driver written without the seam is very hard to retrofit with one; a driver
//! written with it costs nothing extra. [`RetrievalHook`] is deliberately mechanism-agnostic — it
//! says *when* retrieval may occur and *what it may touch*, not how retrieved data is combined.
//! Chunked cross-attention, residual-stream gating, hashed n-gram lookup, and learned index
//! routing are all implementations of this one trait.
//!
//! See `docs/decisions/0006-retrieval-fusion-seam.md` for the evidence behind that choice,
//! including the evidence against.

mod hook;

pub use hook::{ForwardContext, FusionPoint, NoopRetrievalHook, RetrievalHook};
