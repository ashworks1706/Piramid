//! Attention key/value cache.
//!
//! Separate from [`piramid_collections::cache`] on purpose. That module caches *stored vectors and metadata*,
//! keyed by document id, living as long as a collection. A KV cache holds *per-sequence attention
//! state*, keyed by request, living as long as one generation and sized in device memory. Sharing
//! a module would force one eviction policy onto two unrelated lifetimes.
//!
//! Paged allocation is the expected design: fixed-size blocks so concurrent sequences share device
//! memory without fragmentation.
//!
//! Skeleton: no implementation yet.
