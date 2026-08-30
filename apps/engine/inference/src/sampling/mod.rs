//! Turning logits into tokens: greedy, temperature, top-k, top-p, repetition penalties.
//!
//! Separate from [`crate::forward`] because sampling is per-request configuration while the
//! forward pass is per-model, and because it is the easiest part of the stack to test alone.
//!
//! Skeleton: no implementation yet.
