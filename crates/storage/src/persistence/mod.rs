//! Sidecar and mmap helpers.

mod file;
mod index;
mod metadata;
mod mmap;

pub use file::warm_file;
pub use index::{get_wal_path, load_index, save_index, EntryPointer};
pub use metadata::{load_metadata, save_metadata};
pub use mmap::{create_mmap, ensure_file_size, grow_mmap_if_needed, warm_mmap};
