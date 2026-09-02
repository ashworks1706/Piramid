use uuid::Uuid;

use super::super::collection::Collection;
use super::read::get;
use crate::limits;
use piramid_core::error::Result;
use piramid_core::metadata::Metadata;
use piramid_database::storage::record_store::RecordStore;
use piramid_database::storage::wal::WalEntry;

pub fn update_metadata(collection: &mut Collection, id: &Uuid, metadata: Metadata) -> Result<bool> {
    if let Some(entry) = get(collection, id)? {
        let mut wal_entry = WalEntry::Update {
            id: *id,
            vector: entry.vector().to_vec(),
            text: entry.text.clone(),
            metadata: metadata.clone(),
            seq: 0,
        };
        collection.checkpoint.wal.log(&mut wal_entry)?;

        let mut entry = entry;
        entry.metadata = metadata.clone();
        let bytes = RecordStore::encode_document(&entry)?;

        limits::enforce_single(collection, bytes.len())?;
        let index_entry = collection.record_store.append(&bytes)?;
        collection.index.insert(*id, index_entry);
        collection.cache.put_metadata(*id, metadata);
        collection
            .manifest
            .update_vector_count(collection.index.len());
        collection.track_operation()?;
        Ok(true)
    } else {
        Ok(false)
    }
}
