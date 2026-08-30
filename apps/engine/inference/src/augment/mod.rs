//! Where retrieval enters the forward pass.
//!
//! RAG concatenates retrieved text into the prompt: retrieval finishes before generation starts,
//! and the evidence costs context tokens every query. The alternative is letting the model
//! consult an index during the forward pass, so evidence arrives through hidden state instead.
//!
//! [`RetrievalHook`] is the seam for that, deliberately mechanism-agnostic: it says when
//! retrieval may happen and what it may touch, not how the result is combined. Chunked
//! cross-attention, residual-stream gating, hashed n-gram lookup and learned index routing are
//! all implementations of the one trait. `docs/decisions/0006-retrieval-fusion-seam.md` has the
//! evidence, including the evidence against.
//!
//! The trait exists before anything calls it because a forward-pass driver written without the
//! seam is hard to retrofit and one written with it costs nothing. [`NoopRetrievalHook`] is both
//! the default and the control arm for any benchmark claiming a strategy helps.
//!
//! An implementation does not live here. A real strategy queries an index, so it depends on
//! `piramid-search` — and this crate must not, or a collection would stop being queryable with no
//! model loaded. A strategy is its own crate depending on both. `scripts/check-deps.sh` fails if
//! this crate ever gains a retrieval dependency.

mod hook;

pub use hook::{ForwardContext, NoopRetrievalHook, RetrievalHook, RetrievalPoint};
