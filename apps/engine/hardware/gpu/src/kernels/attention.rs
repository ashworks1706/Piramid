//! Attention kernels, reserved for `piramid-inference`.
//!
//! Here rather than in `inference/` because attention is device code with the same lifetime and
//! module-loading concerns as any other kernel. Empty by design; the first occupant should be
//! plain scaled-dot-product attention, the baseline a fused retrieval variant has to beat.
