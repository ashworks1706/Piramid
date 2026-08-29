//! Contiguous vector storage.
//!
//! # Why this exists
//!
//! Vectors are held as `HashMap<Uuid, Vec<f32>>` in several places. That layout makes every vector
//! a separate heap allocation, which has two costs:
//!
//! - **SIMD**: rows are scattered, so the prefetcher cannot stride across candidates.
//! - **Device transfer**: a scattered set cannot be uploaded. Any GPU path must first gather rows
//!   into a contiguous staging buffer on *every call*, and that gather generally costs more than
//!   the kernel saves.
//!
//! A [`VectorSlab`] stores all rows in one `Vec<f32>` with a fixed stride, so a candidate set is a
//! subslice — directly SIMD-friendly, and uploadable in one transfer.
//!
//! # Status
//!
//! The type is complete; nothing is migrated onto it yet. Adopting it behind
//! [`VectorReader::as_slab`](crate::vectors::VectorReader::as_slab) is the intended next
//! step, and can happen one call site at a time because that method is optional.

use std::collections::HashMap;

use uuid::Uuid;

/// Dense row-major storage for equal-length vectors.
///
/// Rows are addressed by a `u32` ordinal rather than a `Uuid`, so hot structures can hold a 4-byte
/// handle instead of a 16-byte key.
#[derive(Debug, Clone, Default)]
pub struct VectorSlab {
    /// All rows, row-major, `len() * dim` elements.
    data: Vec<f32>,
    /// Row width. Zero only while the slab is empty.
    dim: usize,
    /// Ordinal for each id.
    ordinals: HashMap<Uuid, u32>,
    /// Id for each ordinal, parallel to the rows in `data`.
    ids: Vec<Uuid>,
}

impl VectorSlab {
    /// Create an empty slab of `dim`-wide vectors.
    pub fn new(dim: usize) -> Self {
        Self {
            data: Vec::new(),
            dim,
            ordinals: HashMap::new(),
            ids: Vec::new(),
        }
    }

    /// Create an empty slab with room for `capacity` rows.
    pub fn with_capacity(dim: usize, capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(dim * capacity),
            dim,
            ordinals: HashMap::with_capacity(capacity),
            ids: Vec::with_capacity(capacity),
        }
    }

    /// Row width.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Number of rows.
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Whether the slab holds no rows.
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// The whole backing slab, row-major.
    ///
    /// This is what a device upload copies and what a batch kernel indexes.
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// Append a vector, returning its ordinal.
    ///
    /// Returns `None` if `vector.len()` disagrees with [`VectorSlab::dim`], or if `id` is already
    /// present — use [`VectorSlab::replace`] to overwrite.
    pub fn push(&mut self, id: Uuid, vector: &[f32]) -> Option<u32> {
        if vector.len() != self.dim || self.ordinals.contains_key(&id) {
            return None;
        }
        let ordinal = self.ids.len() as u32;
        self.data.extend_from_slice(vector);
        self.ids.push(id);
        self.ordinals.insert(id, ordinal);
        Some(ordinal)
    }

    /// Overwrite the row for `id` in place, returning its ordinal.
    ///
    /// Returns `None` if `id` is absent or the width disagrees.
    pub fn replace(&mut self, id: &Uuid, vector: &[f32]) -> Option<u32> {
        if vector.len() != self.dim {
            return None;
        }
        let ordinal = *self.ordinals.get(id)?;
        let start = ordinal as usize * self.dim;
        self.data[start..start + self.dim].copy_from_slice(vector);
        Some(ordinal)
    }

    /// Ordinal for `id`, if present.
    pub fn ordinal(&self, id: &Uuid) -> Option<u32> {
        self.ordinals.get(id).copied()
    }

    /// Id at `ordinal`, if in range.
    pub fn id_at(&self, ordinal: u32) -> Option<Uuid> {
        self.ids.get(ordinal as usize).copied()
    }

    /// Row for `ordinal`, if in range.
    pub fn row(&self, ordinal: u32) -> Option<&[f32]> {
        let start = (ordinal as usize).checked_mul(self.dim)?;
        self.data.get(start..start + self.dim)
    }

    /// Row for `id`, if present.
    pub fn get(&self, id: &Uuid) -> Option<&[f32]> {
        self.row(self.ordinal(id)?)
    }

    /// Iterate rows in ordinal order.
    pub fn iter(&self) -> impl Iterator<Item = (Uuid, &[f32])> + '_ {
        self.ids
            .iter()
            .copied()
            .zip(self.data.chunks_exact(self.dim.max(1)))
    }

    /// Copy the rows for `ids` into `out`, row-major.
    ///
    /// This is the fallback path for building a contiguous candidate set out of a non-contiguous
    /// source. `out` must be exactly `ids.len() * dim` long. Returns `None` on a length mismatch or
    /// an unknown id.
    pub fn gather_into(&self, ids: &[Uuid], out: &mut [f32]) -> Option<()> {
        if out.len() != ids.len() * self.dim {
            return None;
        }
        for (id, slot) in ids.iter().zip(out.chunks_exact_mut(self.dim)) {
            slot.copy_from_slice(self.get(id)?);
        }
        Some(())
    }
}
