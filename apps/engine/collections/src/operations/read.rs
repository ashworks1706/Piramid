use uuid::Uuid;

use super::super::collection::Collection;
use piramid_core::error::Result;
use piramid_database::storage::document::Document;

pub fn get(storage: &Collection, id: &Uuid) -> Result<Option<Document>> {
    let Some(index_entry) = storage.index.get(id) else {
        return Ok(None);
    };
    storage.record_store.read_document(index_entry).map(Some)
}
