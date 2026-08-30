//! Attention key/value cache.
//!
//! Separate from `piramid-collections::cache`: that's keyed by document id for a collection's
//! lifetime, this is keyed by request for one generation in device memory — different lifetimes,
//! different eviction policy. Expected design: paged, fixed-size blocks so concurrent sequences
//! share memory without fragmenting it.
//!
//! Skeleton: no implementation yet.
