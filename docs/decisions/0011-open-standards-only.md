# 0011 — Open standards only, no vendor integrations

**Context.** `piramid-observability` shipped three exporters: a Prometheus endpoint, OTLP traces,
and a Sentry client. The Sentry integration came from a pattern borrowed from an application
codebase, where error tracking belongs. This is a database.

No database ships a Sentry client. Postgres, ClickHouse, and Qdrant expose metrics and structured
logs and stop there. Sentry is an application error-tracking product, and an operator does not
route their database's errors into it — they collect the process's logs like they do for every
other piece of infrastructure they run.

There is a real constraint pulling the other way. Piramid is a binary, not a library: an operator
cannot recompile it to add a `tracing` layer of their own. Whatever telemetry interfaces exist have
to be built in. So "that is up to them" only works if what they get is something they can point at
their own tooling.

**Decision.** Build in open standards; integrate with no vendor's product.

| Interface | Standard | Status |
|---|---|---|
| `/metrics` | Prometheus text exposition | always on, no feature flag |
| Structured logs on stdout | `LOG_JSON=true` | always available |
| Panics and backtraces on stderr | the Rust runtime | free |
| Distributed traces | OTLP | behind `otel` |

Those four cover what an operator needs, and every one of them is a format rather than a company.
An operator points OTLP at Tempo, Honeycomb, Jaeger, or Axiom from one configuration; they pipe
stdout to Loki, journald, or CloudWatch. If a future exporter is a *protocol*, it is in scope. If
it is a *product*, it is not.

The Sentry integration is removed: `SentryConfig`, the `sentry` feature, the layer, and the two
dependencies.

**Consequences.** Fourteen fewer dependencies under `--all-features`, and the crate no longer makes
an application-monitoring decision on the operator's behalf. Errors are not lost — a panic still
carries a backtrace to stderr, where whatever collects logs already picks it up.

**Not decided.** Whether OTLP is worth keeping yet. It is vendor-neutral and an operator genuinely
cannot add it themselves, but Piramid is one process with no distributed calls, so the spans
currently restate what `/metrics` and the logs already say. Its value arrives when inference lands
and a request spans retrieval, fusion, and a forward pass. It is feature-gated and off by default,
so it costs nothing meanwhile.
