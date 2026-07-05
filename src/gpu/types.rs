use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuError {
    Unavailable(String),
    InvalidInput(String),
    Runtime(String),
}

impl Display for GpuError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable(message) => write!(f, "GPU unavailable: {message}"),
            Self::InvalidInput(message) => write!(f, "GPU invalid input: {message}"),
            Self::Runtime(message) => write!(f, "GPU runtime error: {message}"),
        }
    }
}

impl std::error::Error for GpuError {}

pub trait GpuBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn cosine_similarity_batch(
        &self,
        query: &[f32],
        candidates: &[Vec<f32>],
    ) -> std::result::Result<Vec<f32>, GpuError>;
    fn dot_product_batch(
        &self,
        query: &[f32],
        candidates: &[Vec<f32>],
    ) -> std::result::Result<Vec<f32>, GpuError>;
    fn euclidean_distance_batch(
        &self,
        query: &[f32],
        candidates: &[Vec<f32>],
    ) -> std::result::Result<Vec<f32>, GpuError>;
}
