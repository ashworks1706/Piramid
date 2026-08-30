//! The forward-pass driver.
//!
//! The loop that turns a prompt into tokens: embed, run decoder layers, sample, repeat. The only
//! caller of [`crate::augment::RetrievalHook`], and it should invoke the hook at every
//! [`RetrievalPoint`](crate::augment::RetrievalPoint) the hook accepts.
//!
//! Write it with those call sites in place from the first commit, even while the only
//! implementation is [`NoopRetrievalHook`](crate::augment::NoopRetrievalHook). Adding them later
//! means restructuring the loop.
//!
//! Skeleton: no implementation yet.
