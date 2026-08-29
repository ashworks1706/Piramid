//! Request admission and batch assembly.
//!
//! Owns the scheduler that groups concurrent generation requests into device-efficient batches,
//! including continuous batching — admitting new sequences into a batch already in flight as older
//! ones finish.
//!
//! Sits above [`crate::forward`]: it decides *what runs together*, the forward pass decides
//! *what happens per step*.
//!
//! Skeleton: no implementation yet.
