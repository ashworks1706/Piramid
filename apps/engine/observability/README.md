# piramid-observability

Telemetry export. `piramid_core::stats` measures; this crate ships.

| Exporter | Feature | Variable | Default |
|---|---|---|---|
| OTLP traces | `otel` | `PIRAMID_OTLP_ENDPOINT` | off |
| Sentry errors | `sentry` | `PIRAMID_SENTRY_DSN` | off |
| Prometheus metrics | — | always at `/metrics` | on |
| Span timings in logs | — | `PIRAMID_LOG_SPANS` | off |

OTLP is the wire format rather than a vendor SDK, so Axiom, Grafana Tempo, Honeycomb, and Jaeger
all work from the same configuration.

```bash
cargo build --features otel,sentry
PIRAMID_OTLP_ENDPOINT=http://localhost:4317 piramid serve
```

## Spans

The service layer is instrumented: `search`, `range_search`, `search_by_text`, `insert`, `upsert`,
`delete_vectors`, `embed`, `rebuild_index`, and `compact` each open a span carrying the fields an
operator needs to explain a slow request without reproducing it — collection, `k`, index type,
per-request recall overrides, result count, elapsed time.

A span only reaches the console when an event fires inside it, so a clean search prints nothing.
`PIRAMID_LOG_SPANS=true` adds a line when each operation closes:

```
search{collection=docs k=2 batch=1 index_type=Flat results=1 elapsed_ms=0}: close time.busy=14.6µs
```

That works with no collector and no feature flag, which matters because most operators will never
run OTLP.

`init` returns a guard that must live until shutdown — dropping it flushes pending spans. It logs
an `observability_ready` line reporting what actually resolved, so a variable that did not take
effect is visible at startup rather than silently doing nothing.

Exporter setup failures are logged and skipped, never propagated: telemetry that cannot reach a
collector is not a reason to stop serving queries. A variable set on a build without its feature
warns at startup rather than being silently ignored.
