use piramid_core::metadata::Metadata;
use uuid::Uuid;

/// A search result: the stored entry plus its similarity score.
#[derive(Debug, Clone)]
pub struct Hit {
    pub id: Uuid,
    pub score: f32,
    pub text: String,
    pub vector: Vec<f32>,
    pub metadata: Metadata,
}
