//! Persistence primitives: records, WAL, sidecars, mmap, and vector layout.

pub mod document;
pub mod manifest;
pub mod persistence;
pub mod record_store;
pub mod vectors;
pub mod wal;

pub use document::Document;
pub use manifest::CollectionMetadata;
pub use persistence::SidecarManager;
pub use vectors::{HashMapVectorReader, VectorReader};
