# piramid-server

HTTP transport, use-case services, and shared process state.

## Three layers, and what separates them

The split that is easy to mistake for duplication — `services` here versus
`collections::operations` one crate down. They share nothing:

| Layer | Owns | Example, for one insert |
|---|---|---|
| `http/handlers` | axum extraction only | pull `State`, `Path`, `Json` out of the request |
| `services` | everything true because a *server* exists | shutdown check, read-only check, name validation, DTO → `Document`, **take the write lock**, record lock-wait and latency, enforce the cache budget, build the response |
| `collections::operations` | durability | WAL, record store, offset index, cache, ANN index |

So `services` is the request-scoped layer: locks, metrics, admission, and the API shapes. Delete
it and those move into either the handler — which would then need `AppState`, locks and metrics —
or into `collections`, which would then know about HTTP and shutdown.

`state.rs` holds `AppState`, the composition root: the managers, the config, and the flags every
request reads. `disk.rs` watches free space and flips read-only mode. `cluster` is node identity
and routing, local-only today and on the roadmap for a decision.

`http::ApiError` is where a transport-agnostic `ErrorKind` becomes an HTTP status. That's also what
keeps `piramid-core` free of axum.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
