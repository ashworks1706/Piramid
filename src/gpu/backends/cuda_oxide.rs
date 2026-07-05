use crate::gpu::{GpuBackend, GpuError};

#[derive(Debug, Default)]
pub struct CudaOxideBackend;

impl CudaOxideBackend {
    pub fn new() -> Self {
        Self
    }

    fn unavailable() -> GpuError {
        GpuError::Unavailable(
            "cuda-oxide backend scaffold exists but kernels are not wired yet".to_string(),
        )
    }
}

impl GpuBackend for CudaOxideBackend {
    fn name(&self) -> &'static str {
        "cuda-oxide"
    }

    fn is_available(&self) -> bool {
        false
    }

    fn cosine_similarity_batch(
        &self,
        _query: &[f32],
        _candidates: &[Vec<f32>],
    ) -> std::result::Result<Vec<f32>, GpuError> {
        Err(Self::unavailable())
    }

    fn dot_product_batch(
        &self,
        _query: &[f32],
        _candidates: &[Vec<f32>],
    ) -> std::result::Result<Vec<f32>, GpuError> {
        Err(Self::unavailable())
    }

    fn euclidean_distance_batch(
        &self,
        _query: &[f32],
        _candidates: &[Vec<f32>],
    ) -> std::result::Result<Vec<f32>, GpuError> {
        Err(Self::unavailable())
    }
}
