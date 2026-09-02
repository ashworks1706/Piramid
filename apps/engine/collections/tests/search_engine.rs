#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "assertions in tests"
)]

use std::fs;
use {
    piramid_collections::Collection, piramid_core::metadata::metadata,
    piramid_core::metadata::Filter, piramid_core::Document, piramid_hardware::compute::Metric,
    piramid_retrieval::search::SearchParams,
};

fn cleanup(path: &str) {
    let sidecars = [
        format!("{path}.offsets.db"),
        format!("{path}.wal.db"),
        format!("{path}.vecindex.db"),
        format!("{path}.manifest.db"),
    ];
    for p in std::iter::once(path.to_string()).chain(sidecars) {
        let _ = fs::remove_file(p);
    }
}

#[test]
fn search_respects_filter() {
    let test_db = concat!(env!("CARGO_TARGET_TMPDIR"), "/test_search_filter.db");
    cleanup(test_db);

    {
        let mut storage = Collection::open(test_db).unwrap();

        let e1 = Document::with_metadata(
            vec![1.0, 0.0, 0.0],
            "rust doc".to_string(),
            metadata([("lang", "rust".into())]),
        );
        let e2 = Document::with_metadata(
            vec![0.9, 0.1, 0.0],
            "python doc".to_string(),
            metadata([("lang", "python".into())]),
        );

        storage.insert(e1).unwrap();
        storage.insert(e2).unwrap();

        let filter = Filter::new().eq("lang", "rust");
        let params = SearchParams {
            mode: storage.config().execution,
            filter: Some(&filter),
            filter_overfetch_override: None,
            search_config_override: None,
            min_score: None,
        };

        let results = storage
            .search(&[1.0, 0.0, 0.0], 5, Metric::Cosine, params)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document.text, "rust doc");
    }

    cleanup(test_db);
}

// Flat sorts by the index's configured metric, but the engine rescores with the metric the
// request asked for. Ranking used to run only when a filter was present, so a query using a
// different metric than the collection was built with came back ordered by the wrong one.
#[test]
fn results_are_ranked_by_the_requested_metric_not_the_index_metric() {
    let path = concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/test_rank_by_request_metric.db"
    );
    for suffix in ["", ".offsets.db", ".wal.db", ".vecindex.db", ".manifest.db"] {
        let _ = fs::remove_file(format!("{path}{suffix}"));
    }
    let mut collection = Collection::open(path).unwrap();

    // Cosine ranks these near-identically; dot product orders them by magnitude.
    for (vector, text) in [
        (vec![1.0, 0.0], "unit"),
        (vec![10.0, 1.0], "large"),
        (vec![0.5, 0.0], "small"),
    ] {
        collection
            .insert(Document::new(vector, text.to_string()))
            .unwrap();
    }

    let hits = collection
        .search(&[1.0, 0.0], 3, Metric::DotProduct, SearchParams::default())
        .unwrap();

    assert_eq!(hits.len(), 3);
    assert_eq!(
        hits[0].document.text, "large",
        "highest dot product must rank first"
    );
    assert!(
        hits.windows(2).all(|w| w[0].score >= w[1].score),
        "scores must descend: {:?}",
        hits.iter().map(|h| h.score).collect::<Vec<_>>()
    );
}

// A range query asks "everything scoring at least this", not "of the top k, those that qualify".
// The threshold reaches the engine now, so it narrows the candidate set before `k` truncates
// rather than after — which is what lets an approximate index surface enough qualifying hits.
#[test]
fn a_range_query_fills_k_from_the_whole_qualifying_set() {
    let path = concat!(env!("CARGO_TARGET_TMPDIR"), "/test_range_threshold.db");
    for suffix in ["", ".offsets.db", ".wal.db", ".vecindex.db", ".manifest.db"] {
        let _ = fs::remove_file(format!("{path}{suffix}"));
    }
    let mut collection = Collection::open(path).unwrap();

    // A shallow ramp: the first five clear 0.99, the rest do not.
    for i in 0..10 {
        let angle = i as f32 * 0.03;
        collection
            .insert(Document::new(
                vec![angle.cos(), angle.sin()],
                format!("doc{i}"),
            ))
            .unwrap();
    }

    let params = SearchParams {
        min_score: Some(0.99),
        ..SearchParams::default()
    };
    let hits = collection
        .search(&[1.0, 0.0], 3, Metric::Cosine, params)
        .unwrap();

    assert_eq!(hits.len(), 3, "k filled from everything that qualifies");
    for hit in &hits {
        assert!(
            hit.score >= 0.99,
            "{} scored {}",
            hit.document.text,
            hit.score
        );
    }
    assert!(hits.windows(2).all(|w| w[0].score >= w[1].score));
}
