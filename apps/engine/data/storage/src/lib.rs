//! Persistence primitives: records, WAL, sidecars, mmap, and vector layout.
//!
//! Byte-level primitives only. API behavior, search semantics and collection lifecycle are
//! decided above.

pub mod document;
pub mod manifest;
pub mod persistence;
pub mod quantization;
pub mod record_store;
pub mod vectors;
pub mod wal;

pub use document::Document;
pub use manifest::CollectionMetadata;
pub use quantization::QuantizedVector;
pub use vectors::{HashMapVectorReader, SlabVectorReader, VectorReader, VectorSlab};
