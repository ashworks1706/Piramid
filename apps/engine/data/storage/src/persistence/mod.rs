//! Sidecar and mmap helpers.

mod file;
mod manifest;
mod mmap;
mod offsets;

pub use file::warm_file;
pub use manifest::{load_metadata, save_metadata};
pub use mmap::{create_mmap, ensure_file_size, grow_mmap_if_needed, mapped_or_file_len, warm_mmap};
pub use offsets::{get_wal_path, load_index, save_index, EntryPointer};
