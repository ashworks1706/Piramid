//! The database: where vectors live, how they are found, and the object that owns both.
//!
//! Three layers, in dependency order. [`storage`] is bytes — records, the write-ahead log, mmap,
//! sidecars — and knows nothing about collections. [`index`] and [`search`] are how those bytes
//! are found: traversal structures, and the planning, filtering, scoring and ranking over them.
//! Everything else here is the collection itself, the object that composes a record store, its
//! caches, a checkpoint policy and an index into one queryable thing.
//!
//! They share a crate because the layering is a cycle otherwise: a collection is built on search,
//! search is built on storage, and storage is where a collection's bytes live.

pub mod cache;
pub mod index;
pub mod search;
pub mod storage;

mod collection;
mod document;

pub use cache::{CacheManager, MetadataCache, VectorStore};
pub use collection::{
    compact, find_duplicates, CheckpointManager, Collection, CollectionHandle, CollectionManager,
    CollectionOpenOptions, CompactStats,
};
pub use search::DuplicatePair;
