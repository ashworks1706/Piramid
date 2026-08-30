//! This collection's in-memory state, behind one entry point: [`CacheManager`].
//!
//! Two different things live here, and the split is the point. [`VectorStore`] is *resident*
//! working state — the ANN indexes resolve candidate ids through it, so evicting an entry breaks
//! search. [`MetadataCache`] is a real cache: bounded, evicted oldest-first, safe to drop.

mod manager;
mod metadata_cache;
mod vector_store;

pub use manager::CacheManager;
pub use metadata_cache::MetadataCache;
pub use vector_store::VectorStore;
