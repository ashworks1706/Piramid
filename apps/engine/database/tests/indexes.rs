#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "assertions in tests"
)]

use piramid_database::index::{
    FlatConfig, FlatIndex, HashMapVectorReader, HnswConfig, HnswIndex, IndexConfig, IndexKind,
    IndexSearchRequest, IvfConfig, IvfIndex, VectorIndex,
};
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn flat_index_searches() {
    let mut idx = FlatIndex::new(FlatConfig::default());
    let mut vectors = HashMap::new();

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let v1 = vec![1.0, 0.0, 0.0];
    let v2 = vec![0.0, 1.0, 0.0];
    vectors.insert(id1, v1.clone());
    vectors.insert(id2, v2.clone());
    let reader = HashMapVectorReader::new(&vectors);

    idx.insert(id1, &v1, &reader).unwrap();
    let bootstrap_stats = idx.stats();
    assert_eq!(bootstrap_stats.total_vectors, 1);

    idx.insert(id2, &v2, &reader).unwrap();
    let ready_stats = idx.stats();
    assert_eq!(ready_stats.total_vectors, 2);

    let empty_meta: HashMap<Uuid, piramid_core::metadata::Metadata> = HashMap::new();
    let results = idx
        .search(IndexSearchRequest::new(
            &v1,
            1,
            &reader,
            piramid_core::config::SearchConfig::default(),
            &empty_meta,
        ))
        .unwrap();
    assert_eq!(results.first(), Some(&id1));
}

#[test]
fn hnsw_tombstone_tracks() {
    let mut idx = HnswIndex::new(HnswConfig::default());
    let mut vectors = HashMap::new();

    let id = Uuid::new_v4();
    let vec = vec![1.0, 2.0, 3.0];
    vectors.insert(id, vec.clone());
    let reader = HashMapVectorReader::new(&vectors);
    idx.insert(id, &vec, &reader).unwrap();

    let empty_meta: HashMap<Uuid, piramid_core::metadata::Metadata> = HashMap::new();
    let results = idx.search(&vec, 1, 50, &reader, None, &empty_meta).unwrap();
    assert!(!results.is_empty());

    idx.remove(&id);
    let stats = idx.stats();
    assert_eq!(stats.tombstones, 1);
    assert_eq!(stats.total_nodes, 0);
}

#[test]
fn ivf_search_basic() {
    let config = IvfConfig {
        num_clusters: 2,
        ..IvfConfig::default()
    };
    let mut idx = IvfIndex::new(config);
    let mut vectors = HashMap::new();

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let v1 = vec![1.0, 0.0, 0.0];
    let v2 = vec![0.9, 0.1, 0.0];
    vectors.insert(id1, v1.clone());
    {
        let reader = HashMapVectorReader::new(&vectors);
        idx.insert(id1, &v1, &reader).unwrap();
    }
    let bootstrap_stats = idx.stats();
    assert_eq!(bootstrap_stats.total_vectors, 1);

    vectors.insert(id2, v2.clone());
    let reader = HashMapVectorReader::new(&vectors);
    idx.insert(id2, &v2, &reader).unwrap();
    let ready_stats = idx.stats();
    assert_eq!(ready_stats.total_vectors, 2);

    let empty_meta: HashMap<Uuid, piramid_core::metadata::Metadata> = HashMap::new();
    let results = idx
        .search(IndexSearchRequest::new(
            &v1,
            1,
            &reader,
            piramid_core::config::SearchConfig::default(),
            &empty_meta,
        ))
        .unwrap();
    assert!(!results.is_empty());
}

#[test]
fn ivf_search_fails_before_clusters_are_ready() {
    let config = IvfConfig {
        num_clusters: 4,
        ..IvfConfig::default()
    };
    let mut idx = IvfIndex::new(config);
    let mut vectors = HashMap::new();

    let id = Uuid::new_v4();
    let vec = vec![1.0, 0.0, 0.0];
    vectors.insert(id, vec.clone());
    let reader = HashMapVectorReader::new(&vectors);
    idx.insert(id, &vec, &reader).unwrap();
    assert_eq!(idx.stats().total_vectors, 1);

    let empty_meta: HashMap<Uuid, piramid_core::metadata::Metadata> = HashMap::new();
    let result = idx.search(IndexSearchRequest::new(
        &vec,
        1,
        &reader,
        piramid_core::config::SearchConfig::default(),
        &empty_meta,
    ));
    assert!(result.is_err());

    idx.remove(&id);
    assert_eq!(idx.stats().total_vectors, 0);
}

#[test]
fn ivf_duplicate_insert_uses_id_map_without_duplicate_membership() {
    let config = IvfConfig {
        num_clusters: 2,
        ..IvfConfig::default()
    };
    let mut idx = IvfIndex::new(config);
    let mut vectors = HashMap::new();

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let v1 = vec![1.0, 0.0, 0.0];
    let v2 = vec![0.0, 1.0, 0.0];

    vectors.insert(id1, v1.clone());
    {
        let reader = HashMapVectorReader::new(&vectors);
        idx.insert(id1, &v1, &reader).unwrap();
        idx.insert(id1, &v1, &reader).unwrap();
    }
    assert_eq!(idx.stats().total_vectors, 1);

    vectors.insert(id2, v2.clone());
    let reader = HashMapVectorReader::new(&vectors);
    idx.insert(id2, &v2, &reader).unwrap();
    idx.insert(id2, &v2, &reader).unwrap();

    let stats = idx.stats();
    assert_eq!(stats.total_vectors, 2);
    match stats.details {
        piramid_database::index::IndexDetails::Ivf {
            vectors_per_cluster,
            ..
        } => {
            let indexed_memberships: usize = vectors_per_cluster.iter().sum();
            assert_eq!(indexed_memberships, 2);
        }
        other => panic!("expected IVF stats, got {other:?}"),
    }
}

#[test]
fn index_selector_prefers_expected_types() {
    let cfg = IndexConfig::default();
    assert_eq!(cfg.select_type(1_000), IndexKind::Flat);
    assert_eq!(cfg.select_type(50_000), IndexKind::Ivf);
    assert_eq!(cfg.select_type(500_000), IndexKind::Hnsw);
}

// HNSW evaluates filters mid-traversal, unlike flat which post-filters. Every other filter test
// in the workspace runs against a small collection, which auto-selects flat — so this is the only
// coverage of `passes_filter` on the graph path.
#[test]
fn hnsw_search_applies_a_filter_during_traversal() {
    use piramid_core::metadata::{metadata, Filter, Metadata};

    let mut idx = HnswIndex::new(HnswConfig::default());
    let mut vectors = HashMap::new();
    let mut metadatas: HashMap<Uuid, Metadata> = HashMap::new();

    let mut wanted = Vec::new();
    for i in 0..40 {
        let id = Uuid::new_v4();
        let vector = vec![i as f32 * 0.01, 1.0, 0.0];
        let lang = if i % 2 == 0 { "rust" } else { "go" };
        if lang == "rust" {
            wanted.push(id);
        }
        vectors.insert(id, vector);
        metadatas.insert(id, metadata([("lang", lang.into())]));
    }

    let reader = HashMapVectorReader::new(&vectors);
    for (id, vector) in &vectors {
        idx.insert(*id, vector, &reader).unwrap();
    }

    let filter = Filter::new().eq("lang", "rust");
    let hits = idx
        .search(
            &[0.0, 1.0, 0.0],
            10,
            200,
            &reader,
            Some(&filter),
            &metadatas,
        )
        .unwrap();

    assert!(!hits.is_empty(), "filter must not exclude everything");
    for id in &hits {
        assert!(wanted.contains(id), "a 'go' document survived the filter");
    }

    // No filter reaches strictly more of the graph than a filtered search.
    let unfiltered = idx
        .search(&[0.0, 1.0, 0.0], 10, 200, &reader, None, &metadatas)
        .unwrap();
    assert!(unfiltered.len() >= hits.len());
}
