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

mod checkpoint;
mod collection;
mod compact;
mod document;
mod limits;
mod manager;
mod near_duplicates;
mod open;
mod search_target;

pub use cache::{CacheManager, MetadataCache, VectorStore};
pub use checkpoint::CheckpointManager;
pub use collection::Collection;
pub use compact::{compact, CompactStats};
pub use manager::{CollectionHandle, CollectionManager};
pub use near_duplicates::find_duplicates;
pub use open::CollectionOpenOptions;
pub use search::DuplicatePair;
