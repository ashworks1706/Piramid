//! Input validation for vectors, text, names, and batch sizes.
//!
//! Runs at the service boundary so everything below can assume well-formed input, which is why
//! the compute kernels assert on dimension rather than returning a `Result`.

use crate::error::{Result, ServerError};

/// Reject empty vectors and any non-finite component.
pub fn validate_vector(vector: &[f32]) -> Result<()> {
    if vector.is_empty() {
        return Err(ServerError::InvalidRequest("Vector cannot be empty".to_string()).into());
    }

    for (i, &value) in vector.iter().enumerate() {
        if value.is_nan() {
            return Err(
                ServerError::InvalidRequest(format!("Vector contains NaN at index {}", i)).into(),
            );
        }
        if value.is_infinite() {
            return Err(ServerError::InvalidRequest(format!(
                "Vector contains Infinity at index {}",
                i
            ))
            .into());
        }
    }

    Ok(())
}

/// Validate every vector in a batch.
pub fn validate_vectors(vectors: &[Vec<f32>]) -> Result<()> {
    for (i, vector) in vectors.iter().enumerate() {
        validate_vector(vector)
            .map_err(|e| ServerError::InvalidRequest(format!("Vector at index {}: {}", i, e)))?;
    }
    Ok(())
}

/// Scale a vector to unit length.
///
/// With normalized vectors, dot product and cosine similarity coincide, and magnitude stops
/// influencing ranking. A zero or non-finite magnitude returns a zero vector rather than NaN.
pub fn normalize_vector(vector: &[f32]) -> Vec<f32> {
    let magnitude: f32 = vector.iter().map(|&x| x * x).sum::<f32>().sqrt();

    if magnitude == 0.0 || magnitude.is_nan() || magnitude.is_infinite() {
        return vec![0.0; vector.len()];
    }

    vector.iter().map(|&x| x / magnitude).collect()
}

/// Check a vector against the collection's dimensionality.
pub fn validate_dimensions(vector: &[f32], expected_dim: usize) -> Result<()> {
    if vector.len() != expected_dim {
        return Err(ServerError::InvalidRequest(format!(
            "Vector dimension mismatch: expected {}, got {}",
            expected_dim,
            vector.len()
        ))
        .into());
    }
    Ok(())
}

/// Reject text above the size limit.
pub fn validate_text(text: &str) -> Result<()> {
    if text.is_empty() {
        return Err(ServerError::InvalidRequest("Text cannot be empty".to_string()).into());
    }

    if text.len() > 1_000_000 {
        return Err(ServerError::InvalidRequest(format!(
            "Text too large: {} bytes (max 1MB)",
            text.len()
        ))
        .into());
    }

    Ok(())
}

/// Collection names are used as filename stems, so they are restricted to characters that are
/// safe on every supported platform.
pub fn validate_collection_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(
            ServerError::InvalidRequest("Collection name cannot be empty".to_string()).into(),
        );
    }

    if name.len() > 255 {
        return Err(ServerError::InvalidRequest(
            "Collection name too long (max 255 chars)".to_string(),
        )
        .into());
    }

    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ServerError::InvalidRequest(
            "Collection name can only contain alphanumeric characters, underscores, and hyphens"
                .to_string(),
        )
        .into());
    }

    Ok(())
}

/// Reject batches above `max`.
pub fn validate_batch_size(size: usize, max_size: usize, operation: &str) -> Result<()> {
    if size == 0 {
        return Err(
            ServerError::InvalidRequest(format!("{} batch cannot be empty", operation)).into(),
        );
    }

    if size > max_size {
        return Err(ServerError::InvalidRequest(format!(
            "{} batch too large: {} items (max {})",
            operation, size, max_size
        ))
        .into());
    }

    Ok(())
}
