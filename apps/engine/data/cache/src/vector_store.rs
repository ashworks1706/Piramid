//! The resident vector map every index reads through.

use std::collections::HashMap;

use piramid_storage::vectors::VectorReader;
use uuid::Uuid;

/// All of a collection's vectors, resident in memory.
///
/// Not a cache: the ANN indexes hold ids and resolve them here, so an evicted entry is a search
/// failure, not a slower search. Bounding memory happens by evicting the [`MetadataCache`]
/// (crate::MetadataCache), never this. The planned `VectorSlab` migration replaces the backing
/// map, not this contract.
#[derive(Default)]
pub struct VectorStore {
    vectors: HashMap<Uuid, Vec<f32>>,
}

impl VectorStore {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace the vector for `id`.
    pub fn put(&mut self, id: Uuid, vector: Vec<f32>) {
        self.vectors.insert(id, vector);
    }

    /// Remove the vector for `id`.
    pub fn remove(&mut self, id: &Uuid) {
        self.vectors.remove(id);
    }

    /// Drop every vector. Only correct before a rebuild repopulates the store.
    pub fn clear(&mut self) {
        self.vectors.clear();
    }

    /// The backing map, keyed by id.
    pub fn vectors(&self) -> &HashMap<Uuid, Vec<f32>> {
        &self.vectors
    }

    /// Approximate resident bytes.
    pub fn usage_bytes(&self) -> usize {
        self.vectors
            .values()
            .map(|vector| std::mem::size_of::<Uuid>() + vector.len() * std::mem::size_of::<f32>())
            .sum()
    }
}

impl VectorReader for VectorStore {
    fn get(&self, id: &Uuid) -> Option<&[f32]> {
        self.vectors.get(id).map(Vec::as_slice)
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (Uuid, &'a [f32])> + 'a> {
        Box::new(
            self.vectors
                .iter()
                .map(|(id, vector)| (*id, vector.as_slice())),
        )
    }

    fn len(&self) -> usize {
        self.vectors.len()
    }
}
