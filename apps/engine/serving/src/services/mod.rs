pub mod admin;
pub mod api;
pub mod collection;
pub mod convert;
pub mod embedding;
pub mod vector;

// Error strings shared by several services.
pub const VECTOR_NOT_FOUND: &str = "Vector not found";
pub const EMBEDDING_NOT_CONFIGURED: &str = "Embedding service not configured";
