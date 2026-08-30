use crate::cache::CacheManager;
use piramid_core::Result;

use super::collection::Collection;
use super::operations;

pub fn rebuild(collection: &mut Collection) -> Result<()> {
    let mut cache = CacheManager::new(collection.config.cache);
    for id in collection.index.keys() {
        if let Some(entry) = operations::get(collection, id)? {
            cache.put_vector(*id, entry.vector().to_vec());
            cache.put_metadata(*id, entry.metadata.clone());
        }
    }
    collection.cache = cache;
    Ok(())
}
