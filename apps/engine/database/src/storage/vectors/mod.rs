//! Vector layout and access, separately from [crate::storage::record_store].

pub mod reader;

pub use reader::{HashMapVectorReader, VectorReader, VectorSlab};
