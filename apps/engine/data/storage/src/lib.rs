//! Persistence primitives: records, WAL, sidecars, mmap, and vector layout.
//!
//! Storage never decides API behavior, search semantics, or collection lifecycle. It provides
//! safe byte-level primitives for the domain layer above it.

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
