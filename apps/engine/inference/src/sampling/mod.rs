//! Turning logits into tokens.
//!
//! Greedy, temperature, top-k, top-p, and the repetition penalties that go with them. Kept separate
//! from [`crate::forward`] because sampling strategy is per-request configuration while
//! the forward pass is per-model, and because sampling is the easiest part of the stack to test in
//! isolation.
//!
//! Skeleton: no implementation yet.
