//! Vector layout and access, separately from [`crate::record_store`].

pub mod reader;

pub use reader::{HashMapVectorReader, VectorReader};
