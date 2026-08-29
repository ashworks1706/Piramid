# piramid-observability

Telemetry export. `piramid-core::telemetry` measures; this crate ships.

| Exporter | Feature | Variable | Default |
|---|---|---|---|
| OTLP traces | `otel` | `PIRAMID_OTLP_ENDPOINT` | off |
| Sentry errors | `sentry` | `PIRAMID_SENTRY_DSN` | off |
| Prometheus metrics | — | always at `/metrics` | on |

OTLP is the wire format rather than a vendor SDK, so Axiom, Grafana Tempo, Honeycomb, and Jaeger
all work from the same configuration.

```bash
cargo build --features otel,sentry
PIRAMID_OTLP_ENDPOINT=http://localhost:4317 piramid serve
```

`init` returns a guard that must live until shutdown — dropping it flushes pending spans.

Exporter setup failures are logged and skipped, never propagated: telemetry that cannot reach a
collector is not a reason to stop serving queries. A variable set on a build without its feature
warns at startup rather than being silently ignored.
