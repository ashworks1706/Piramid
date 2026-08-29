//! Quantization and dequantization kernels.
//!
//! Reserved for on-device precision conversion — `f32` to `f16`/`bf16`/int8 and back. Serves both
//! stored vectors and model weights, which is why it sits beside the other kernels rather than
//! inside `piramid-storage::quantization`; that module owns the *format*, this one owns the device code
//! that converts it.
