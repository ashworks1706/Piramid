//! Attention kernels.
//!
//! Reserved for [`piramid_inference`]. Kept in `gpu/` rather than `inference/` because attention is
//! device code with the same lifetime and module-loading concerns as every other kernel; the
//! inference layer calls in through a typed wrapper the same way the retrieval path does.
//!
//! Empty by design. The first occupant is expected to be a standard scaled-dot-product attention
//! kernel, which is the baseline any fused retrieval variant has to beat.
