//! The seam where retrieval enters the forward pass.
//!
//! Defined apart from both halves. An implementation depends on this crate, on retrieval and on
//! model; none of them depend on it.

mod hook;

pub use hook::{
    ForwardContext, HiddenState, NoopPending, NoopRetrievalHook, PendingRetrieval, RetrievalHook,
    RetrievalPoint, RetrievalRequest,
};
