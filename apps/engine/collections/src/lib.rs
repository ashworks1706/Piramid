pub mod cache;

mod builder;
mod checkpoint;
mod collection;
mod compact;
mod dup;
mod manager;
mod operations;
mod search;

pub use builder::CollectionBuilder;
pub use cache::{CacheManager, MetadataCache, VectorStore};
pub use checkpoint::CheckpointManager;
pub use collection::Collection;
pub use compact::{compact, CompactStats};
pub use dup::{find_duplicates, DuplicateHit};
pub use manager::{CollectionHandle, CollectionManager};

#[derive(Clone, Default)]
pub struct CollectionOpenOptions {
    pub config: piramid_core::config::CollectionConfig,
}

impl From<piramid_core::config::CollectionConfig> for CollectionOpenOptions {
    fn from(config: piramid_core::config::CollectionConfig) -> Self {
        Self { config }
    }
}

use piramid_core::error::Result;
use piramid_database::metadata::Metadata;
use piramid_database::storage::document::Document;
use piramid_hardware::compute::Metric;
use piramid_retrieval::search::Hit;
use std::collections::HashMap;
use uuid::Uuid;

impl Collection {
    pub fn open(path: &str) -> Result<Self> {
        CollectionBuilder::open(path, CollectionOpenOptions::default())
    }

    pub fn open_with_options(path: &str, options: CollectionOpenOptions) -> Result<Self> {
        CollectionBuilder::open(path, options)
    }

    pub fn get(&self, id: &Uuid) -> Result<Option<Document>> {
        operations::get(self, id)
    }

    pub fn insert(&mut self, entry: Document) -> Result<Uuid> {
        operations::insert(self, entry)
    }

    pub fn insert_batch(&mut self, entries: Vec<Document>) -> Result<Vec<Uuid>> {
        operations::insert_batch(self, entries)
    }

    pub fn upsert(&mut self, entry: Document) -> Result<Uuid> {
        operations::upsert(self, entry)
    }

    pub fn delete(&mut self, id: &Uuid) -> Result<bool> {
        operations::delete(self, id)
    }

    pub fn delete_batch(&mut self, ids: &[Uuid]) -> Result<usize> {
        operations::delete_batch(self, ids)
    }

    pub fn update_metadata(&mut self, id: &Uuid, metadata: Metadata) -> Result<bool> {
        operations::update_metadata(self, id, metadata)
    }

    pub fn update_vector(&mut self, id: &Uuid, vector: Vec<f32>) -> Result<bool> {
        operations::update_vector(self, id, vector)
    }

    pub fn search(
        &self,
        query: &[f32],
        k: usize,
        metric: Metric,
        params: piramid_retrieval::search::SearchParams,
    ) -> Result<Vec<Hit>> {
        search::search(self, query, k, metric, params)
    }

    pub fn search_batch_with(
        &self,
        queries: &[Vec<f32>],
        k: usize,
        metric: Metric,
        params: piramid_retrieval::search::SearchParams,
    ) -> Result<Vec<Vec<Hit>>> {
        search::search_batch(self, queries, k, metric, params)
    }

    pub fn get_vectors(&self) -> &HashMap<Uuid, Vec<f32>> {
        self.vectors_view()
    }

    pub fn checkpoint(&mut self) -> Result<()> {
        checkpoint::checkpoint(self)
    }

    pub fn flush(&mut self) -> Result<()> {
        checkpoint::flush(self)
    }
}
