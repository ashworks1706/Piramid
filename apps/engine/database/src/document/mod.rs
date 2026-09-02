//! What can be done to one document in a collection.

mod metadata;
mod read;
mod write;

pub use metadata::update_metadata;
pub use read::get;
pub use write::{
    delete, delete_batch, delete_internal, insert, insert_batch, insert_internal, update_vector,
    upsert,
};
