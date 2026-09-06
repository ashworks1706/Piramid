//! Renders /api/metrics JSON into the Prometheus text exposition format.

use piramid_core::observability::prometheus::{MetricType, Registry};

use crate::services::api::MetricsResponse;

/// Render a metrics snapshot in the Prometheus text format.
pub fn render(metrics: &MetricsResponse) -> String {
    let mut registry = Registry::new();

    registry.metric(
        "piramid_collections_total",
        "Number of collections currently loaded.",
        MetricType::Gauge,
        metrics.total_collections as f64,
    );
    registry.metric(
        "piramid_vectors_total",
        "Number of vectors across all loaded collections.",
        MetricType::Gauge,
        metrics.total_vectors as f64,
    );

    let by_collection = |extract: fn(&crate::services::api::CollectionMetrics) -> Option<f64>| {
        metrics
            .collections
            .iter()
            .filter_map(|c| extract(c).map(|value| (vec![("collection", c.name.clone())], value)))
            .collect::<Vec<_>>()
    };

    registry.metric_family(
        "piramid_collection_vectors",
        "Vectors in a collection.",
        MetricType::Gauge,
        by_collection(|c| Some(c.vector_count as f64)),
    );
    registry.metric_family(
        "piramid_collection_memory_bytes",
        "Approximate resident bytes for a collection.",
        MetricType::Gauge,
        by_collection(|c| Some(c.memory_usage_bytes as f64)),
    );
    registry.metric_family(
        "piramid_collection_insert_latency_ms",
        "Mean insert latency in milliseconds.",
        MetricType::Gauge,
        by_collection(|c| c.insert_latency_ms.map(f64::from)),
    );
    registry.metric_family(
        "piramid_collection_search_latency_ms",
        "Mean search latency in milliseconds.",
        MetricType::Gauge,
        by_collection(|c| c.search_latency_ms.map(f64::from)),
    );
    registry.metric_family(
        "piramid_collection_lock_read_ms",
        "Mean time waiting for a collection read lock, in milliseconds.",
        MetricType::Gauge,
        by_collection(|c| c.lock_read_ms.map(f64::from)),
    );
    registry.metric_family(
        "piramid_collection_lock_write_ms",
        "Mean time waiting for a collection write lock, in milliseconds.",
        MetricType::Gauge,
        by_collection(|c| c.lock_write_ms.map(f64::from)),
    );

    // The index type is published as a label on a constant-1 gauge.
    registry.metric_family(
        "piramid_collection_index_info",
        "Index family in use for a collection.",
        MetricType::Gauge,
        metrics
            .collections
            .iter()
            .map(|c| {
                (
                    vec![
                        ("collection", c.name.clone()),
                        ("index_type", c.index_type.clone()),
                    ],
                    1.0,
                )
            })
            .collect::<Vec<_>>(),
    );

    registry.metric_family(
        "piramid_wal_size_bytes",
        "Write-ahead log size for a collection.",
        MetricType::Gauge,
        metrics
            .wal_stats
            .iter()
            .filter_map(|w| {
                w.wal_size_bytes
                    .map(|bytes| (vec![("collection", w.collection.clone())], bytes as f64))
            })
            .collect::<Vec<_>>(),
    );
    registry.metric_family(
        "piramid_wal_checkpoint_age_seconds",
        "Seconds since a collection last checkpointed.",
        MetricType::Gauge,
        metrics
            .wal_stats
            .iter()
            .filter_map(|w| {
                w.checkpoint_age_secs
                    .map(|age| (vec![("collection", w.collection.clone())], age as f64))
            })
            .collect::<Vec<_>>(),
    );

    registry.metric(
        "piramid_embedding_requests_total",
        "Embedding requests issued to the provider.",
        MetricType::Counter,
        metrics.embedding.requests as f64,
    );
    registry.metric(
        "piramid_embedding_texts_total",
        "Texts submitted for embedding.",
        MetricType::Counter,
        metrics.embedding.texts as f64,
    );
    registry.metric(
        "piramid_embedding_tokens_total",
        "Tokens reported by the embedding provider.",
        MetricType::Counter,
        metrics.embedding.total_tokens as f64,
    );
    if let Some(latency) = metrics.embedding.avg_latency_ms {
        registry.metric(
            "piramid_embedding_latency_ms",
            "Mean embedding request latency in milliseconds.",
            MetricType::Gauge,
            f64::from(latency),
        );
    }

    registry.render()
}
