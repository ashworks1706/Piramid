//! Near-duplicate detection.

use std::collections::HashSet;
use uuid::Uuid;

use super::collection::Collection;
use piramid_compute::{strategies::for_mode, Metric};
use piramid_core::error::Result;
use piramid_index::IndexSearchRequest;

/// Neighbours examined per document when the caller does not say.
const DEFAULT_NEIGHBOR_K: usize = 49;

#[derive(Debug)]
pub struct DuplicateHit {
    pub id_a: Uuid,
    pub id_b: Uuid,
    pub score: f32,
}

pub fn find_duplicates(
    collection: &Collection,
    metric: Metric,
    threshold: f32,
    limit: Option<usize>,
    k_override: Option<usize>,
    ef_override: Option<usize>,
    nprobe_override: Option<usize>,
) -> Result<Vec<DuplicateHit>> {
    let mut pairs = Vec::new();
    let vectors = collection.vectors_view();
    let metadatas = collection.metadata_view();
    let ids: Vec<Uuid> = vectors.keys().cloned().collect();
    let kernels = for_mode(collection.config.execution)?;
    let mut search_cfg = collection.config.search;
    if let Some(ef) = ef_override {
        search_cfg.ef = Some(ef);
    }
    if let Some(nprobe) = nprobe_override {
        search_cfg.nprobe = Some(nprobe);
    }
    // One source for the neighbour count, clamped to what the collection can actually supply.
    let neighbor_k = k_override
        .unwrap_or(DEFAULT_NEIGHBOR_K)
        .min(ids.len().saturating_sub(1))
        .max(1);

    let mut seen = HashSet::new();

    for id in &ids {
        let Some(vec) = vectors.get(id) else {
            continue;
        };
        let neighbors = collection.vector_index().search(IndexSearchRequest::new(
            vec,
            neighbor_k,
            collection.vector_reader(),
            search_cfg,
            metadatas,
        ))?;
        for neighbor_id in neighbors {
            if neighbor_id == *id {
                continue;
            }
            let (a, b) = if id < &neighbor_id {
                (*id, neighbor_id)
            } else {
                (neighbor_id, *id)
            };
            if !seen.insert((a, b)) {
                continue;
            }
            if let (Some(va), Some(vb)) = (vectors.get(&a), vectors.get(&b)) {
                let score = metric.calculate(va, vb, kernels);
                if score >= threshold {
                    pairs.push(DuplicateHit {
                        id_a: a,
                        id_b: b,
                        score,
                    });
                }
            }
        }
    }

    pairs.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if let Some(max) = limit {
        pairs.truncate(max);
    }
    Ok(pairs)
}
