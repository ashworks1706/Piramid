//! The forward-pass driver.
//!
//! Owns the loop that turns a prompt into tokens: embed, run decoder layers, sample, repeat. It is
//! the only caller of [`crate::fusion::RetrievalHook`], and is expected to invoke the
//! hook at every [`FusionPoint`](crate::fusion::FusionPoint) the hook accepts.
//!
//! Build this with the hook call sites in place from the first commit, even while the only
//! implementation is [`NoopRetrievalHook`](crate::fusion::NoopRetrievalHook). Adding
//! them afterwards means restructuring the loop.
//!
//! Skeleton: no implementation yet.
