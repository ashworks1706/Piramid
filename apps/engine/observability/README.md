# piramid-observability

Where measurements go. `piramid_core::stats` records them; this crate ships them.

| What | Feature | Variable | Default |
|---|---|---|---|
| Prometheus metrics | none | served at `/metrics` | on |
| Span timings in logs | none | `PIRAMID_LOG_SPANS` | off |
| OTLP traces | `otel` | `PIRAMID_OTLP_ENDPOINT` | off |

OTLP is a wire format rather than a vendor SDK, so Axiom, Grafana Tempo, Honeycomb, and Jaeger all
work from the same configuration. That's the line this crate holds: open standards only, and no
integration with any vendor's product. Errors reach an operator as panics and `tracing` events on
stderr, which whatever collects their logs already picks up. See ADR 0011.

```bash
cargo build --features otel
PIRAMID_OTLP_ENDPOINT=http://localhost:4317 piramid serve
```

## Spans

The service layer is instrumented. `search`, `range_search`, `search_by_text`, `insert`, `upsert`,
`delete_vectors`, `embed`, `rebuild_index`, and `compact` each open a span carrying what you need
to explain a slow request without reproducing it: collection, `k`, index type, per-request recall
overrides, result count, elapsed time.

A span only reaches the console when an event fires inside it, so a clean search prints nothing.
`PIRAMID_LOG_SPANS=true` adds a line when each operation closes:

```
search{collection=docs k=2 batch=1 index_type=Flat results=1 elapsed_ms=0}: close time.busy=14.6µs
```

That needs no collector and no feature flag, which matters because most operators will never run
OTLP.

`init` returns a guard that has to live until shutdown, since dropping it flushes pending spans. It
logs an `observability_ready` line saying what actually resolved, so a variable that didn't take
effect shows up at startup rather than silently doing nothing.

Exporter setup failures are logged and skipped rather than propagated. Telemetry that can't reach a
collector isn't a reason to stop serving queries.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
