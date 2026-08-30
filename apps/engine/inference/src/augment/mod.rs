//! Where retrieval enters the forward pass.
//!
//! [`RetrievalHook`] is the seam: mechanism-agnostic, defined before anything calls it. See
//! `docs/decisions/0006-retrieval-fusion-seam.md`. An implementation lives in its own crate,
//! never here — `scripts/check-deps.sh` enforces that this crate stays free of retrieval deps.

mod hook;

pub use hook::{ForwardContext, NoopRetrievalHook, RetrievalHook, RetrievalPoint};
