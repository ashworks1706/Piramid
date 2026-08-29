# piramid-core

Shared foundation: errors, configuration, metadata and filters, validation, telemetry.

The vocabulary every other crate shares. Transport-agnostic by rule — `PiramidError` exposes an
`ErrorKind`, never an HTTP status — and it never ends the process: configuration loading returns
`Result` so the binary decides.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
