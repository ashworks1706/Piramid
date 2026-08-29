//! Retrieval fused into the forward pass.
//!
//! This module is the reason the rest of the crate is shaped the way it is.
//!
//! Conventional RAG concatenates retrieved text into the prompt: retrieval finishes before
//! generation starts, and the evidence costs context-window tokens. Fusion instead lets the model
//! consult the index *during* the forward pass, so evidence enters through hidden state rather
//! than through the token stream.
//!
//! [`RetrievalHook`] is the seam where that happens. It is defined now, ahead of any code that can
//! call it, for one reason: a forward-pass driver written without the seam is extremely hard to
//! retrofit with one, and a driver written with it costs nothing extra. The trait is deliberately
//! mechanism-agnostic — it says *when* retrieval may occur and *what it may touch*, not how the
//! retrieved data is combined. Chunked cross-attention, residual-stream gating, and learned index
//! routing are all implementations of this one trait.

mod hook;

pub use hook::{ForwardContext, FusionPoint, NoopRetrievalHook, RetrievalHook};
