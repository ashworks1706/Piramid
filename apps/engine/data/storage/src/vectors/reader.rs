//! Vector access abstraction: how indexes read vectors they do not own.

use std::collections::HashMap;

use uuid::Uuid;

use crate::vectors::slab::VectorSlab;

/// Read-only access to a collection's vectors.
pub trait VectorReader: Sync {
    /// Vector for `id`, if present.
    fn get(&self, id: &Uuid) -> Option<&[f32]>;

    /// Iterate every `(id, vector)` pair.
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (Uuid, &'a [f32])> + 'a>;

    /// Number of vectors available.
    fn len(&self) -> usize;

    /// Whether no vectors are available.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Row width, if the reader holds a uniform-width set.
    fn dim(&self) -> Option<usize> {
        self.iter().next().map(|(_, vector)| vector.len())
    }

    /// The whole vector set as one contiguous row-major `(data, dim)` slice, if stored that way.
    fn as_slab(&self) -> Option<(&[f32], usize)> {
        None
    }

    /// Copies the vectors for `ids` into `out`, row-major; `out` must be `ids.len() * dim` long.
    fn gather_into(&self, ids: &[Uuid], out: &mut [f32]) -> Option<()> {
        let dim = self.dim()?;
        if out.len() != ids.len() * dim {
            return None;
        }
        for (id, slot) in ids.iter().zip(out.chunks_exact_mut(dim)) {
            slot.copy_from_slice(self.get(id)?);
        }
        Some(())
    }
}

/// A [`VectorReader`] over a scattered `HashMap` of owned vectors.
pub struct HashMapVectorReader<'a> {
    vectors: &'a HashMap<Uuid, Vec<f32>>,
}

impl<'a> HashMapVectorReader<'a> {
    /// Borrow a map of vectors as a reader.
    pub fn new(vectors: &'a HashMap<Uuid, Vec<f32>>) -> Self {
        Self { vectors }
    }
}

impl VectorReader for HashMapVectorReader<'_> {
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

/// A [`VectorReader`] over a contiguous [`VectorSlab`].
pub struct SlabVectorReader<'a> {
    slab: &'a VectorSlab,
}

impl<'a> SlabVectorReader<'a> {
    /// Borrow a slab as a reader.
    pub fn new(slab: &'a VectorSlab) -> Self {
        Self { slab }
    }
}

impl VectorReader for SlabVectorReader<'_> {
    fn get(&self, id: &Uuid) -> Option<&[f32]> {
        self.slab.get(id)
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (Uuid, &'a [f32])> + 'a> {
        Box::new(self.slab.iter())
    }

    fn len(&self) -> usize {
        self.slab.len()
    }

    fn dim(&self) -> Option<usize> {
        Some(self.slab.dim())
    }

    fn as_slab(&self) -> Option<(&[f32], usize)> {
        Some((self.slab.data(), self.slab.dim()))
    }

    fn gather_into(&self, ids: &[Uuid], out: &mut [f32]) -> Option<()> {
        self.slab.gather_into(ids, out)
    }
}
