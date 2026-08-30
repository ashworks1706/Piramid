//! Model execution backends.
//!
//! Same containment rule as `piramid-gpu::backends`: a framework's types stay in its own module,
//! the rest of the crate sees only traits. `candle` is the initial target — Rust-native, and it
//! shares the CUDA device retrieval already uses.

#[cfg(feature = "inference-candle")]
pub mod candle;
