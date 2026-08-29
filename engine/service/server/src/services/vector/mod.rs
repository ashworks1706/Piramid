use crate::runtime::SharedState;
use piramid_core::error::{Result, ServerError};

mod read;
mod search;
mod write;

pub use read::{get_vector, list_vectors};
pub use search::{range_search_vectors, search_vectors};
pub use write::{delete_vector, delete_vectors, insert_vector, upsert_vector};

pub(super) const MAX_BATCH_SIZE: usize = 10_000;

pub(super) fn ensure_available(state: &SharedState) -> Result<()> {
    if state
        .shutting_down
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        return Err(ServerError::ServiceUnavailable("Server is shutting down".to_string()).into());
    }
    Ok(())
}
