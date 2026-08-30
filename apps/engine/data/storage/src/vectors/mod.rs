//! Vector layout and access, separately from [`crate::record_store`], which owns document layout
//! on disk.
//!
//! Layout is a storage concern, not an index concern: several indexes read the same vectors, and
//! the layout is what decides whether a device path is viable at all.

pub mod reader;
pub mod slab;

pub use reader::{HashMapVectorReader, SlabVectorReader, VectorReader};
pub use slab::VectorSlab;
