pub mod admin;
pub mod collection;
pub mod convert;
pub mod embedding;
pub mod types;
pub mod vector;

// Error strings shared by several services, so the same condition reads the same in every
// response.
pub const VECTOR_NOT_FOUND: &str = "Vector not found";
pub const EMBEDDING_NOT_CONFIGURED: &str = "Embedding service not configured";
