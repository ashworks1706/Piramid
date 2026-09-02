# piramid-core

Shared vocabulary: every error the app wraps, the whole configuration surface, document metadata
and the filters over it, input validation, the counters the engine keeps about itself, and where
those counters go.

Transport-agnostic. `PiramidError` exposes an `ErrorKind` — `NotFound`, `Conflict`,
`Upstream` — and never a status code; mapping onto HTTP is `serving`'s job.

`config` is one file with two blocks, split by when a setting takes effect: `startup` is fixed
at boot, `runtime` is re-read on reload. Unknown keys and unimplemented settings fail at startup
rather than being ignored.

`observability` owns telemetry end to end: `install` turns a `LoggingConfig` into a subscriber —
including which per-subsystem directives it implies — and `prometheus` encodes the text exposition
format. The binary decides *when* to call it and nothing more. OTLP export is behind the `otel`
feature, so a default build pulls in no exporter. `stats` is the other half: plain atomics with no
exporter dependency, so any crate can record into it.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
