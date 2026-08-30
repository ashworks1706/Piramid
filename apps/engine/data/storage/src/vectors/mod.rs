//! Vector layout and access, separately from [`crate::record_store`].

pub mod reader;
pub mod slab;

pub use reader::{HashMapVectorReader, SlabVectorReader, VectorReader};
pub use slab::VectorSlab;
