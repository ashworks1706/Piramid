//! The resident vector slab every index reads through.

use std::collections::HashMap;

use piramid_core::error::{Result, ServerError};
use uuid::Uuid;

use crate::storage::vectors::VectorReader;

/// All of a collection's vectors, resident in memory as one contiguous buffer.
///
/// Not a cache: the ANN indexes hold ids and resolve them here, so an evicted entry is a search
/// failure, not a slower search. Bounding memory happens by evicting the
/// [`MetadataCache`](crate::MetadataCache), never this.
///
/// One `Vec<f32>` at a fixed stride, not a map of owned rows. That is what a prefetcher can stride
/// and what a device takes in one `cudaMemcpy`; a map of `Vec`s is one allocation per vector and
/// forces a gather before any batch kernel can run. Ids resolve through a `Uuid -> u32` ordinal
/// map, so hot structures can hold a 4-byte handle rather than a 16-byte key.
///
/// Ordinals are stable. A removed row becomes a hole rather than being filled by moving the last
/// row into it, because a moved row would invalidate every index adjacency list referencing it —
/// and repairing those costs more than the hole. Holes are reused by the next insert.
#[derive(Default)]
pub struct VectorStore {
    /// Row-major, `dim` floats per row. Holes are still allocated; their contents are stale.
    slab: Vec<f32>,
    /// Row width, fixed by the first vector stored and cleared only by [`VectorStore::clear`].
    dim: Option<usize>,
    /// Id to row.
    ordinals: HashMap<Uuid, u32>,
    /// Row to id. `None` marks a hole.
    ids: Vec<Option<Uuid>>,
    /// Holes, reused before the slab grows.
    free: Vec<u32>,
}

impl VectorStore {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Row width, once anything has been stored.
    pub fn dim(&self) -> Option<usize> {
        self.dim
    }

    /// Rows allocated but not live. Every one of them makes [`VectorReader::as_slab`] return
    /// `None`, because a slab with dead rows in it is not the vector set. Inserts reuse them, so
    /// this is only non-zero while deletes are running ahead of inserts.
    pub fn holes(&self) -> usize {
        self.free.len()
    }

    /// Insert or replace the vector for `id`.
    ///
    /// A width other than the store's is an error rather than a resize: every row has to stay at
    /// one stride for the slab to mean anything, and silently padding or truncating would hand a
    /// kernel numbers no caller asked for.
    pub fn put(&mut self, id: Uuid, vector: &[f32]) -> Result<()> {
        let dim = *self.dim.get_or_insert(vector.len());
        if vector.len() != dim {
            return Err(ServerError::InvalidRequest(format!(
                "Vector dimension mismatch: collection holds {dim}, got {}",
                vector.len()
            ))
            .into());
        }
        let ordinal = match self.ordinals.get(&id) {
            Some(existing) => *existing,
            None => self.claim_row(id, dim),
        };
        let start = ordinal as usize * dim;
        self.slab[start..start + dim].copy_from_slice(vector);
        Ok(())
    }

    /// Remove the vector for `id`, leaving its row as a hole.
    pub fn remove(&mut self, id: &Uuid) {
        let Some(ordinal) = self.ordinals.remove(id) else {
            return;
        };
        if let Some(slot) = self.ids.get_mut(ordinal as usize) {
            *slot = None;
        }
        self.free.push(ordinal);
    }

    /// Drop every vector, and the row width with them. Only correct before a rebuild repopulates
    /// the store, which is also the only time the width is allowed to change.
    pub fn clear(&mut self) {
        self.slab.clear();
        self.ids.clear();
        self.free.clear();
        self.ordinals.clear();
        self.dim = None;
    }

    /// Resident bytes: the slab plus the two id maps.
    pub fn usage_bytes(&self) -> usize {
        self.slab.len() * std::mem::size_of::<f32>()
            + self.ordinals.len() * (std::mem::size_of::<Uuid>() + std::mem::size_of::<u32>())
            + self.ids.len() * std::mem::size_of::<Option<Uuid>>()
    }

    /// A hole if there is one, otherwise a new row at the end.
    fn claim_row(&mut self, id: Uuid, dim: usize) -> u32 {
        let ordinal = match self.free.pop() {
            Some(ordinal) => {
                self.ids[ordinal as usize] = Some(id);
                ordinal
            }
            None => {
                let ordinal = u32::try_from(self.ids.len()).unwrap_or(u32::MAX);
                self.ids.push(Some(id));
                self.slab.resize(self.slab.len() + dim, 0.0);
                ordinal
            }
        };
        self.ordinals.insert(id, ordinal);
        ordinal
    }

    fn row(&self, ordinal: u32) -> &[f32] {
        let dim = self.dim.unwrap_or(0);
        let start = ordinal as usize * dim;
        &self.slab[start..start + dim]
    }
}

impl VectorReader for VectorStore {
    fn get(&self, id: &Uuid) -> Option<&[f32]> {
        self.ordinals.get(id).map(|ordinal| self.row(*ordinal))
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (Uuid, &'a [f32])> + 'a> {
        Box::new(
            self.ordinals
                .iter()
                .map(|(id, ordinal)| (*id, self.row(*ordinal))),
        )
    }

    fn len(&self) -> usize {
        self.ordinals.len()
    }

    fn dim(&self) -> Option<usize> {
        self.dim
    }

    /// The whole slab, when every allocated row is live.
    ///
    /// A hole holds a removed vector's stale floats, and a batch kernel scoring the slab has no
    /// way to skip it — so a store with holes reports `None` and the caller gathers instead. The
    /// next insert reuses a hole, and collection compaction rebuilds the store from the record
    /// store, so nothing needs a repack of its own.
    fn as_slab(&self) -> Option<(&[f32], usize)> {
        let dim = self.dim?;
        self.free.is_empty().then_some((self.slab.as_slice(), dim))
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "a failed assertion is the point of a test"
)]
mod tests {
    use super::*;

    fn store(rows: &[(Uuid, [f32; 2])]) -> VectorStore {
        let mut store = VectorStore::new();
        for (id, vector) in rows {
            store.put(*id, vector).unwrap();
        }
        store
    }

    #[test]
    fn rows_are_contiguous_and_readable_by_id() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let store = store(&[(a, [1.0, 2.0]), (b, [3.0, 4.0])]);

        assert_eq!(store.get(&a), Some([1.0, 2.0].as_slice()));
        assert_eq!(store.get(&b), Some([3.0, 4.0].as_slice()));
        assert_eq!(store.len(), 2);
        assert_eq!(VectorReader::dim(&store), Some(2));

        // The whole point: one buffer a device takes in one copy.
        let (slab, dim) = store.as_slab().unwrap();
        assert_eq!(dim, 2);
        assert_eq!(slab, [1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn a_replaced_vector_is_written_in_place_rather_than_appended() {
        let a = Uuid::new_v4();
        let mut store = store(&[(a, [1.0, 2.0])]);

        store.put(a, &[9.0, 9.0]).unwrap();

        assert_eq!(store.len(), 1);
        assert_eq!(store.as_slab().unwrap().0, [9.0, 9.0]);
    }

    #[test]
    fn a_width_that_is_not_the_stride_is_refused() {
        let mut store = store(&[(Uuid::new_v4(), [1.0, 2.0])]);

        // Padding or truncating here would hand a kernel numbers nobody asked for.
        let error = store.put(Uuid::new_v4(), &[1.0, 2.0, 3.0]).unwrap_err();

        assert!(error.to_string().contains("dimension mismatch"), "{error}");
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn a_hole_withdraws_the_slab_and_the_next_insert_fills_it() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let mut store = store(&[(a, [1.0, 2.0]), (b, [3.0, 4.0])]);

        store.remove(&a);

        assert_eq!(store.len(), 1);
        assert_eq!(store.get(&a), None);
        assert_eq!(store.holes(), 1);
        // Row 0 still holds a's stale floats and a batch kernel cannot skip it, so the fast path
        // is withdrawn rather than handing back a slab with a dead row in it.
        assert!(store.as_slab().is_none());
        // b's ordinal did not move: filling the hole by shifting rows down would invalidate every
        // index adjacency list pointing at row 1.
        assert_eq!(store.get(&b), Some([3.0, 4.0].as_slice()));

        let c = Uuid::new_v4();
        store.put(c, &[5.0, 6.0]).unwrap();

        assert_eq!(store.holes(), 0);
        assert_eq!(store.as_slab().unwrap().0, [5.0, 6.0, 3.0, 4.0]);
        assert_eq!(store.get(&c), Some([5.0, 6.0].as_slice()));
    }

    #[test]
    fn iteration_and_gathering_see_only_live_rows() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let mut store = store(&[(a, [1.0, 1.0]), (b, [2.0, 2.0]), (c, [3.0, 3.0])]);
        store.remove(&b);

        let mut seen: Vec<Uuid> = store.iter().map(|(id, _)| id).collect();
        seen.sort();
        let mut expected = vec![a, c];
        expected.sort();
        assert_eq!(seen, expected);

        // The fallback path an index takes while the slab is withdrawn.
        let mut out = [0.0; 4];
        store.gather_into(&[c, a], &mut out).unwrap();
        assert_eq!(out, [3.0, 3.0, 1.0, 1.0]);
    }

    #[test]
    fn clearing_releases_the_stride_so_a_rebuild_can_change_it() {
        let mut store = store(&[(Uuid::new_v4(), [1.0, 2.0])]);

        store.clear();

        assert_eq!(store.len(), 0);
        assert_eq!(VectorReader::dim(&store), None);
        assert!(store.as_slab().is_none());

        // A rebuild is the one time the width is allowed to change.
        store.put(Uuid::new_v4(), &[1.0, 2.0, 3.0]).unwrap();
        assert_eq!(VectorReader::dim(&store), Some(3));
    }

    #[test]
    fn an_empty_store_offers_no_slab_rather_than_an_empty_one() {
        let store = VectorStore::new();

        // dim is unknown until something is stored, and (&[], 0) would divide by zero downstream.
        assert!(store.as_slab().is_none());
        assert!(store.is_empty());
    }
}
