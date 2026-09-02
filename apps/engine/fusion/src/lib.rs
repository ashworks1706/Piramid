#![deny(missing_docs)]

//! The seam where retrieval enters the forward pass.
//!
//! Defined apart from both halves on purpose: a model runtime that depends on the retrieval stack
//! cannot be built without it, and a retrieval stack that depends on a model runtime stops being
//! queryable with no model loaded. An implementation depends on this crate, `retrieval` and
//! `model`; none of them depend on it.

mod hook;

pub use hook::{
    ForwardContext, HiddenState, NoopPending, NoopRetrievalHook, PendingRetrieval, RetrievalHook,
    RetrievalPoint, RetrievalRequest,
};
