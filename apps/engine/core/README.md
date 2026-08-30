# piramid-core

Errors, configuration, metadata and filters, validation, and stats.

The vocabulary every other crate shares.

It's transport-agnostic by rule: `PiramidError` exposes an `ErrorKind`, never an HTTP status. It
also never ends the process, since configuration loading returns a `Result` and the binary decides
what to do with it.

`stats` holds latency, lock, and embedding counters as plain atomics with no dependency on
`tracing` or any exporter, so any crate can record into it. Shipping those numbers anywhere is
`piramid-observability`'s job.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
