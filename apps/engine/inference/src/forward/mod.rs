//! The forward-pass driver.
//!
//! Turns a prompt into tokens: embed, run decoder layers, sample, repeat. The only caller of
//! [`crate::augment::RetrievalHook`] — call sites for every
//! [`RetrievalPoint`](crate::augment::RetrievalPoint) must be in place from the first commit,
//! even with only [`NoopRetrievalHook`](crate::augment::NoopRetrievalHook) behind them, since
//! adding them later means restructuring the loop.
//!
//! Skeleton: no implementation yet.
