//! Quantization and dequantization kernels.
//!
//! On-device precision conversion — `f32` to `f16`/`bf16`/int8 and back — for both stored vectors
//! and model weights. `piramid-storage::quantization` owns the format; this owns the device code.
