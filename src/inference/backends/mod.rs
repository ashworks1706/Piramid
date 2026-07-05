//! Inference backend adapters.
//!
//! Candle is the initial scaffold target for local model execution.

mod candle;

pub use candle::CandleInferenceBackend;
