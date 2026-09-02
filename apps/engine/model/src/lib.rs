//! Model execution and the embedding providers that feed it.
//!
//! [`inference`] is the forward pass and its memory. [`embeddings`] is the provider stack that
//! turns text into vectors, over HTTP today.

pub mod embeddings;
pub mod inference;
