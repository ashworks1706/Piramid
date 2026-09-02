#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "assertions in tests"
)]

use std::fs;
use {
    piramid_collections::Collection, piramid_core::metadata::metadata,
    piramid_core::metadata::Filter, piramid_database::storage::Document,
    piramid_hardware::compute::Metric, piramid_retrieval::search::SearchParams,
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
        };

        let results = storage
            .search(&[1.0, 0.0, 0.0], 5, Metric::Cosine, params)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "rust doc");
    }

    cleanup(test_db);
}
