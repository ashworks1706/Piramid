//! Attention key/value cache.
//!
//! Separate from `piramid-collections::cache` on purpose: that caches stored vectors and metadata
//! keyed by document id for as long as a collection lives, this holds per-sequence attention
//! state keyed by request for one generation, sized in device memory. One module would force one
//! eviction policy onto two unrelated lifetimes.
//!
//! Paged allocation is the expected design: fixed-size blocks, so concurrent sequences share
//! device memory without fragmenting it.
//!
//! Skeleton: no implementation yet.
