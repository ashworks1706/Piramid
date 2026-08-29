//! Model architecture definitions and weight loading.
//!
//! Owns the shape of a model — layer counts, hidden dimensions, vocabulary, tokenizer bindings —
//! and the code that maps a checkpoint on disk into device memory via `piramid-gpu::DeviceBuffer`.
//!
//! Weight *format* concerns (safetensors parsing, dtype conversion) belong here; weight *precision*
//! conversion kernels belong in `piramid-gpu::kernels::quantize`.
//!
//! Skeleton: no implementation yet.
