# piramid-server

HTTP transport, use-case services, and shared runtime state.

Handlers stay thin: parse, call a service, serialize. `http::ApiError` is where a transport-
agnostic `ErrorKind` becomes an HTTP status, which is also what keeps `piramid-core` free of axum.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
