//! Vector layout and access.
//!
//! Owns *how vectors are laid out in memory* and *how they are read*, separately from
//! [`crate::record_store`], which owns how documents are laid out on disk.
//!
//! Layout is a storage concern rather than an index concern because several indexes read the same
//! vectors, and because the layout — not the index — determines whether a device path is viable.

pub mod reader;
pub mod slab;

pub use reader::{HashMapVectorReader, SlabVectorReader, VectorReader};
pub use slab::VectorSlab;
