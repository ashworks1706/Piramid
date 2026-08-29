//! Where retrieval enters the forward pass.
//!
//! Conventional RAG concatenates retrieved text into the prompt: retrieval finishes before
//! generation starts, and the evidence costs context-window tokens on every query. The
//! alternative is to let the model consult an index *during* the forward pass, so evidence
//! enters through hidden state rather than through the token stream.
//!
//! [`RetrievalHook`] is the seam for that. It is deliberately mechanism-agnostic — it says *when*
//! retrieval may occur and *what it may touch*, not how retrieved data is combined. Chunked
//! cross-attention, residual-stream gating, hashed n-gram lookup, and learned index routing are
//! all implementations of one trait. See `docs/decisions/0006-retrieval-fusion-seam.md` for the
//! evidence behind that choice, including the evidence against.
//!
//! # Why the trait exists before anything calls it
//!
//! A forward-pass driver written without the seam is very hard to retrofit with one; a driver
//! written with it costs nothing extra. [`NoopRetrievalHook`] is both the default and the control
//! arm for any benchmark claiming a strategy helps.
//!
//! # Where an implementation lives
//!
//! Not here. A real strategy has to query an index, so it depends on `piramid-search` — and this
//! crate must not, or a collection would stop being queryable without a model loaded. A strategy
//! is therefore its own crate depending on both `piramid-inference` and `piramid-search`:
//!
//! ```text
//! piramid-inference ←── a strategy crate ──→ piramid-search
//! ```
//!
//! `scripts/check-deps.sh` fails if this crate ever gains a retrieval dependency.

mod hook;

pub use hook::{ForwardContext, NoopRetrievalHook, RetrievalHook, RetrievalPoint};
