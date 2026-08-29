//! Model execution backends.
//!
//! Same containment rule as `piramid-gpu::backends`: an execution framework's types stay inside
//! its own module here, and the rest of the crate sees only the traits above.
//!
//! `candle` is the initial target, chosen because it keeps the stack Rust-native and shares the
//! same CUDA device the retrieval path already uses.

#[cfg(feature = "inference-candle")]
pub mod candle;
