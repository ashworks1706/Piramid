//! Sidecar and mmap helpers. [`SidecarManager`] is the entry point.

mod file;
mod manager;
mod mmap;
mod offsets;

pub use file::warm_file;
pub use manager::SidecarManager;
pub(crate) use mmap::{
    create_mmap, ensure_file_size, grow_mmap_if_needed, mapped_or_file_len, warm_mmap,
};
pub use offsets::EntryPointer;
