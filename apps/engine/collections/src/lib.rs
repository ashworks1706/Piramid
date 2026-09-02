//! A collection: the object that owns a record store, its caches, a checkpoint policy and an index.

pub mod cache;

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
pub use piramid_retrieval::search::DuplicatePair;
