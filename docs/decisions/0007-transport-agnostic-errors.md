# 0007 — Errors carry a kind, not a status code

**Context.** `PiramidError` and `ServerError` had `status_code() -> axum::http::StatusCode` and
`impl IntoResponse`. That put `axum` in the dependency list of the crate every other crate depends
on, so splitting the workspace would have dragged an HTTP framework into storage, index, and
compute.

The obvious fix — move `IntoResponse` to the server crate — is blocked by the orphan rule: both
`PiramidError` and `IntoResponse` would be foreign there.

**Decision.** `core` classifies; the transport maps.

`PiramidError::kind()` returns `ErrorKind` (`BadRequest`, `NotFound`, `Conflict`, `Upstream`,
`Unavailable`, `Internal`, …) with no protocol in it. `server::http::ApiError` is a local newtype
that maps a kind onto a status and renders JSON, which sidesteps the orphan rule because the impl
is on a local type.

Handlers keep `?` because `ApiError: From<E> where E: Into<PiramidError>`. A handler returns
`ApiResult<T>`; everything below returns `piramid_core::Result`. Handler signatures did not change
— only the `Result` import.

**Consequences.** `core` has no HTTP dependency, and a second transport (gRPC) maps the same kinds
without touching any library crate. The redundant `http::types` module, which only re-exported
`services::types`, is gone; there is one canonical path per DTO.

**Not decided.** Whether `ErrorKind` grows a machine-readable code for API clients. The current
JSON body carries a message and the numeric status, which is what the existing surface promised.
