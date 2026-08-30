use piramid_compute::{strategies::for_mode, DistanceKernels, Metric};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use uuid::Uuid;

use super::config::{HnswConfig, HnswStats};
use crate::{MetadataReader, VectorReader};
use piramid_core::error::{IndexError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HnswNode {
    /// Neighbours per layer, layer 0 first.
    connections: Vec<Vec<Uuid>>,
    /// Deleted, but edges are kept so traversal stays connected.
    tombstone: bool,
}

#[derive(Debug, Clone)]
struct SearchCandidate {
    id: Uuid,
    distance: f32,
}

struct SearchContext<'a> {
    vectors: &'a dyn VectorReader,
    filter: Option<&'a piramid_core::metadata::Filter>,
    metadatas: &'a dyn MetadataReader,
    /// Resolved once per operation; traversal computes thousands of distances against it.
    kernels: &'a dyn DistanceKernels,
}
impl PartialEq for SearchCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for SearchCandidate {}

impl PartialOrd for SearchCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reversed: BinaryHeap is a max-heap, and we want the closest first.
        other
            .distance
            .partial_cmp(&self.distance)
            .unwrap_or(Ordering::Equal)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HnswIndex {
    config: HnswConfig,
    nodes: HashMap<Uuid, HnswNode>,
    max_level: isize,
    start_node: Option<Uuid>,
}

impl HnswIndex {
    pub fn new(config: HnswConfig) -> Self {
        HnswIndex {
            config,
            nodes: HashMap::new(),
            max_level: -1,
            start_node: None,
        }
    }

    fn is_tombstone(&self, id: &Uuid) -> bool {
        self.nodes.get(id).map(|n| n.tombstone).unwrap_or(false)
    }

    fn mark_tombstone(&mut self, id: &Uuid) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.tombstone = true;
        }
    }
    /// Draw a layer for a new node; exponential decay keeps upper layers sparse.
    fn random_layer(&self) -> usize {
        // floor(-ln(uniform) * ml)
        let r: f32 = rand::random();
        (-r.ln() * self.config.ml).floor() as usize
    }

    /// Insert `id`, linking it into each layer it occupies.
    pub fn insert(&mut self, id: Uuid, vector: &[f32], vectors: &dyn VectorReader) -> Result<()> {
        let kernels = for_mode(self.config.mode)?;
        let empty_meta: HashMap<Uuid, piramid_core::metadata::Metadata> = HashMap::new();
        let search_context = SearchContext {
            vectors,
            filter: None,
            metadatas: &empty_meta,
            kernels,
        };
        let layer = self.random_layer();

        // The first node becomes the entry point and has nothing to link to.
        let entry_point = match self.start_node {
            None => {
                self.start_node = Some(id);
                self.max_level = layer as isize;
                self.nodes.insert(
                    id,
                    HnswNode {
                        connections: vec![Vec::new(); layer + 1],
                        tombstone: false,
                    },
                );
                return Ok(());
            }
            Some(entry_point) => entry_point,
        };

        // Greedy descent from the entry point to find where to link.
        let mut current_entry = vec![entry_point];

        for lc in ((layer as isize + 1)..=self.max_level).rev() {
            current_entry =
                self.search_layer(vector, &current_entry, 1, lc as usize, &search_context);
        }

        // Connect from the target layer down to 0. Connections are staged and applied after
        // pruning so a failure cannot leave a half-linked node.
        let mut pending_connections = vec![Vec::new(); layer + 1];
        for lc in (0..=layer).rev() {
            current_entry = self.search_layer(
                vector,
                &current_entry,
                self.config.ef_construction,
                lc,
                &search_context,
            );

            // Layer 0 allows M_max edges; higher layers allow M.
            let m = if lc == 0 {
                self.config.m_max
            } else {
                self.config.m
            };
            let neighbors = self.select_neighbors(&current_entry, m, vectors, vector, kernels);

            // Edges are undirected, so each link is written in both directions.
            for &neighbor_id in &neighbors {
                if lc < pending_connections.len() {
                    pending_connections[lc].push(neighbor_id);
                }

                if let Some(neighbor) = self.nodes.get_mut(&neighbor_id) {
                    if lc < neighbor.connections.len() {
                        neighbor.connections[lc].push(id);

                        // Degree is capped per node; prune the neighbour if this edge pushed it over.
                        if neighbor.connections[lc].len() > m {
                            // Cloned to release the borrow on `self.nodes` before pruning.
                            let neighbor_connections = neighbor.connections[lc].clone();
                            let neighbor_vec = vectors
                                .get(&neighbor_id)
                                .ok_or_else(|| {
                                    IndexError::SearchFailed(format!(
                                        "HNSW neighbour {neighbor_id} is missing from vector storage"
                                    ))
                                })?
                                .to_vec();

                            let pruned = self.select_neighbors(
                                &neighbor_connections,
                                m,
                                vectors,
                                &neighbor_vec,
                                kernels,
                            );

                            if let Some(neighbor) = self.nodes.get_mut(&neighbor_id) {
                                if lc < neighbor.connections.len() {
                                    neighbor.connections[lc] = pruned;
                                }
                            }
                        }
                    }
                }
            }
        }

        let new_node = HnswNode {
            connections: pending_connections,
            tombstone: false,
        };
        self.nodes.insert(id, new_node);

        // A node above the current entry point becomes the new entry point.
        if layer as isize > self.max_level {
            self.max_level = layer as isize;
            self.start_node = Some(id);
        }
        Ok(())
    }

    /// Find the `k` nearest neighbours of `query`, widening to `ef` candidates at layer 0.
    pub fn search(
        &self,
        query: &[f32],
        k: usize,
        ef: usize,
        vectors: &dyn VectorReader,
        filter: Option<&piramid_core::metadata::Filter>,
        metadatas: &dyn MetadataReader,
    ) -> Result<Vec<Uuid>> {
        let Some(ep) = self.start_node else {
            return Ok(Vec::new());
        };
        let kernels = for_mode(self.config.mode)?;

        if vectors.get(&ep).is_none() {
            return Err(IndexError::SearchFailed(format!(
                "HNSW entry point {ep} is missing from vector storage"
            ))
            .into());
        }
        let mut current_nearest = vec![ep];

        let search_context = SearchContext {
            vectors,
            filter,
            metadatas,
            kernels,
        };

        for lc in (1..=self.max_level as usize).rev() {
            current_nearest = self.search_layer(query, &current_nearest, 1, lc, &search_context);
        }

        current_nearest = self.search_layer(query, &current_nearest, ef.max(k), 0, &search_context);

        let mut filtered: Vec<Uuid> = current_nearest
            .into_iter()
            .filter(|id| !self.is_tombstone(id))
            .collect();
        filtered.truncate(k);
        Ok(filtered)
    }

    /// Search one layer, returning neighbour ids nearest-first.
    fn search_layer(
        &self,
        query: &[f32],
        entry_points: &[Uuid],
        num_closest: usize,
        level: usize,
        context: &SearchContext<'_>,
    ) -> Vec<Uuid> {
        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new();
        let mut nearest = BinaryHeap::new();

        for &ep in entry_points {
            if let Some(ep_vector) = context.vectors.get(&ep) {
                if let Some(f) = context.filter {
                    if let Some(md) = context.metadatas.get(&ep) {
                        if !f.matches(md) {
                            continue;
                        }
                    }
                }
                let dist = self.distance(query, ep_vector, context.kernels);
                candidates.push(SearchCandidate {
                    id: ep,
                    distance: dist,
                });
                if !self.is_tombstone(&ep) {
                    nearest.push(SearchCandidate {
                        id: ep,
                        distance: dist,
                    });
                }
                visited.insert(ep);
            }
        }

        let mut furthest_distance = nearest.peek().map(|c| c.distance).unwrap_or(f32::INFINITY);

        // Explore the closest candidate first, stopping once nothing closer remains.
        while let Some(candidate) = candidates.pop() {
            if candidate.distance > furthest_distance {
                break;
            }

            if let Some(node) = self.nodes.get(&candidate.id) {
                if level < node.connections.len() {
                    for &neighbor_id in &node.connections[level] {
                        if visited.insert(neighbor_id) {
                            if let Some(neighbor_vector) = context.vectors.get(&neighbor_id) {
                                if let Some(f) = context.filter {
                                    if let Some(md) = context.metadatas.get(&neighbor_id) {
                                        if !f.matches(md) {
                                            continue;
                                        }
                                    }
                                }
                                let dist = self.distance(query, neighbor_vector, context.kernels);
                                let neighbor_dead = self.is_tombstone(&neighbor_id);

                                if dist < furthest_distance || nearest.len() < num_closest {
                                    candidates.push(SearchCandidate {
                                        id: neighbor_id,
                                        distance: dist,
                                    });
                                    if !neighbor_dead {
                                        nearest.push(SearchCandidate {
                                            id: neighbor_id,
                                            distance: dist,
                                        });

                                        if nearest.len() > num_closest {
                                            nearest.pop(); // remove furthest
                                        }

                                        furthest_distance = nearest
                                            .peek()
                                            .map(|c| c.distance)
                                            .unwrap_or(f32::INFINITY);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut result: Vec<_> = nearest.into_iter().collect();
        result.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(Ordering::Equal)
        });
        result.into_iter().map(|c| c.id).collect()
    }

    /// Pick the `m` closest candidates by distance only (no diversity heuristic).
    fn select_neighbors(
        &self,
        candidates: &[Uuid],
        m: usize,
        vectors: &dyn VectorReader,
        query: &[f32],
        kernels: &dyn DistanceKernels,
    ) -> Vec<Uuid> {
        if candidates.len() <= m {
            return candidates.to_vec();
        }

        let mut distances: Vec<_> = candidates
            .iter()
            .filter_map(|&id| {
                if self.is_tombstone(&id) {
                    return None;
                }
                vectors.get(&id).map(|vec| {
                    let dist = self.distance(query, vec, kernels);
                    (id, dist)
                })
            })
            .collect();

        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
        distances.truncate(m);
        distances.into_iter().map(|(id, _)| id).collect()
    }

    /// Distance under the configured metric, normalized so smaller is always nearer.
    fn distance(&self, a: &[f32], b: &[f32], kernels: &dyn DistanceKernels) -> f32 {
        let score = self.config.metric.calculate(a, b, kernels);
        match self.config.metric {
            // Similarity metrics score higher for nearer; invert them.
            Metric::Cosine | Metric::DotProduct => 1.0 - score,
            Metric::Euclidean => score,
        }
    }

    /// Tombstone a node, keeping its edges so traversal stays connected.
    pub fn remove(&mut self, id: &Uuid) {
        if !self.nodes.contains_key(id) {
            return;
        }
        self.mark_tombstone(id);

        if self.start_node == Some(*id) {
            self.start_node = self
                .nodes
                .iter()
                .find(|(_, n)| !n.tombstone)
                .map(|(k, _)| *k);
            self.max_level = self
                .nodes
                .values()
                .filter(|n| !n.tombstone)
                .map(|n| n.connections.len() as isize - 1)
                .max()
                .unwrap_or(-1);
        }
    }

    /// Graph shape and approximate memory use.
    pub fn stats(&self) -> HnswStats {
        let mut total_nodes = 0;
        let mut tombstones = 0;
        let mut layer_sizes = vec![0; (self.max_level + 1) as usize];
        let mut total_connections = 0;

        for node in self.nodes.values() {
            if node.tombstone {
                tombstones += 1;
            } else {
                total_nodes += 1;
                for (layer, connections) in node.connections.iter().enumerate() {
                    if layer < layer_sizes.len() {
                        layer_sizes[layer] += 1;
                    }
                    total_connections += connections.len();
                }
            }
        }

        let memory_usage_bytes = self.nodes.len() * std::mem::size_of::<(Uuid, HnswNode)>()
            + self
                .nodes
                .values()
                .map(|n| {
                    n.connections
                        .iter()
                        .map(|c| c.len() * std::mem::size_of::<Uuid>())
                        .sum::<usize>()
                })
                .sum::<usize>();

        HnswStats {
            total_nodes,
            tombstones,
            max_layer: self.max_level,
            layer_sizes,
            memory_usage_bytes,
            avg_connections: if total_nodes > 0 {
                total_connections as f32 / total_nodes as f32
            } else {
                0.0
            },
        }
    }

    /// Configured default for the search-time `ef` knob.
    pub fn get_ef_search(&self) -> usize {
        self.config.ef_search
    }
}
