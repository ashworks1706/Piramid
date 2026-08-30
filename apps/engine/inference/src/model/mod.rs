//! Model architecture definitions and weight loading.
//!
//! The shape of a model — layer counts, hidden dimensions, vocabulary, tokenizer bindings — and
//! the code that maps a checkpoint into device memory via `piramid-gpu::DeviceBuffer`. Weight
//! format belongs here; precision-conversion kernels belong in `piramid-gpu::kernels::quantize`.
//!
//! Skeleton: no implementation yet.
