mod factory;
pub mod ollama;
pub mod openai;

pub use factory::{create_embedder, EmbeddingProvider};
pub use ollama::OllamaEmbedder;
pub use openai::OpenAIEmbedder;
