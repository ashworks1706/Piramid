# piramid-core

Shared foundation: errors, configuration, metadata and filters, validation, self-measurement.

The vocabulary every other crate shares. Transport-agnostic by rule — `PiramidError` exposes an
`ErrorKind`, never an HTTP status — and it never ends the process: configuration loading returns
`Result` so the binary decides.

`stats` holds latency, lock, and embedding counters as plain atomics, with no dependency on
`tracing` or any exporter, so any crate can record into it. Shipping those numbers anywhere is
`piramid-observability`'s job.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
