use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::vault::parser::BlockType;

/// Configuration for embedding models and processors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model_name: String,
    pub max_tokens: usize,
    pub chunk_size: usize,
    pub overlap_size: usize,
    pub dimension: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_name: "all-MiniLM-L6-v2".to_string(),
            max_tokens: 512,
            chunk_size: 512,
            overlap_size: 50,
            dimension: 384,
        }
    }
}

/// Represents a text embedding with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEmbedding {
    pub id: String,
    pub text: String,
    pub embedding: Vec<f32>,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Vector embedding data structure expected by the search engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingVector {
    pub text: String,
    pub vector: Vec<f32>,
    pub model_name: String,
    pub created_at: DateTime<Utc>,
    pub block_embeddings: Option<Vec<BlockEmbedding>>,
}

/// Block-level embedding for fine-grained search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEmbedding {
    pub block_id: String,
    pub embedding: Vec<f32>,
    pub content: String,
    pub block_type: BlockType,
    pub start_pos: usize,
    pub end_pos: usize,
}

/// Trait for embedding providers
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str, model_name: &str) -> Result<Vec<f32>>;
    async fn embed_batch(&self, texts: &[String], model_name: &str) -> Result<Vec<Vec<f32>>>;
}

/// Main embeddings processor that implements EmbeddingProvider
pub struct Embeddings {
    config: EmbeddingConfig,
    cache: HashMap<String, TextEmbedding>,
    cache_dir: PathBuf,
}

#[async_trait]
impl EmbeddingProvider for Embeddings {
    async fn embed(&self, text: &str, _model_name: &str) -> Result<Vec<f32>> {
        info!("Generating embedding for text chunk of {} characters", text.len());
        
        // For now, return a mock embedding vector
        // TODO: Replace with actual model inference in Day 3 implementation
        let embedding = self.generate_mock_embedding(text).await?;
        
        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[String], model_name: &str) -> Result<Vec<Vec<f32>>> {
        info!("Generating embeddings for batch of {} texts", texts.len());
        
        let mut embeddings = Vec::new();
        for text in texts {
            let embedding = self.embed(text, model_name).await?;
            embeddings.push(embedding);
        }
        
        Ok(embeddings)
    }
}

impl Embeddings {
    /// Create a new embeddings processor
    pub fn new() -> Result<Self> {
        let config = EmbeddingConfig::default();
        let cache_dir = PathBuf::from("cache/embeddings");
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir)?;
        }

        Ok(Self {
            config,
            cache: HashMap::new(),
            cache_dir,
        })
    }

    /// Generate embedding for text (stub implementation)
    pub async fn embed_text(&mut self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        info!("Generating embedding for text chunk of {} characters", text.len());
        
        // Check cache first
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let cache_key = format!("{:x}", hasher.finish());
        
        if let Some(cached) = self.cache.get(&cache_key) {
            debug!("Found cached embedding for text");
            return Ok(cached.embedding.clone());
        }

        // For now, return a mock embedding vector
        // TODO: Replace with actual model inference in Day 3 implementation
        let embedding = self.generate_mock_embedding(text).await?;
        
        // Cache the result
        let text_embedding = TextEmbedding {
            id: cache_key.clone(),
            text: text.to_string(),
            embedding: embedding.clone(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
        };
        
        self.cache.insert(cache_key, text_embedding);
        
        Ok(embedding)
    }

    /// Generate embeddings for multiple text chunks
    pub async fn embed_batch(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        info!("Generating embeddings for batch of {} texts", texts.len());
        
        let mut embeddings = Vec::new();
        for text in texts {
            let embedding = self.embed_text(text).await?;
            embeddings.push(embedding);
        }
        
        Ok(embeddings)
    }

    /// Split text into chunks for embedding
    pub fn chunk_text(&self, text: &str) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut chunks = Vec::new();
        let mut current_chunk: Vec<&str> = Vec::new();
        let mut current_length = 0;

        for word in words {
            let word_len = word.len();
            
            if current_length + word_len > self.config.chunk_size && !current_chunk.is_empty() {
                // Create chunk with overlap
                chunks.push(current_chunk.join(" "));
                
                // Keep overlap words
                let overlap_words = if current_chunk.len() > self.config.overlap_size {
                    current_chunk.split_off(current_chunk.len() - self.config.overlap_size)
                } else {
                    current_chunk.clone()
                };
                
                current_chunk = overlap_words;
                current_length = current_chunk.iter().map(|w| w.len()).sum::<usize>();
            }
            
            current_chunk.push(word);
            current_length += word_len + 1; // +1 for space
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk.join(" "));
        }

        debug!("Split text into {} chunks", chunks.len());
        chunks
    }

    /// Save embedding cache to disk
    pub fn save_cache(&self) -> Result<()> {
        let cache_file = self.cache_dir.join("embeddings_cache.json");
        let json = serde_json::to_string_pretty(&self.cache)?;
        
        fs::write(cache_file, json)?;
        info!("Saved {} embeddings to cache", self.cache.len());
        Ok(())
    }

    /// Load embedding cache from disk
    pub fn load_cache(&mut self) -> Result<()> {
        let cache_file = self.cache_dir.join("embeddings_cache.json");
        
        if cache_file.exists() {
            let content = fs::read_to_string(cache_file)?;
            self.cache = serde_json::from_str(&content)?;
            
            info!("Loaded {} embeddings from cache", self.cache.len());
        }
        
        Ok(())
    }

    /// Get embedding configuration
    pub fn config(&self) -> &EmbeddingConfig {
        &self.config
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        let total_embeddings = self.cache.len();
        let total_memory = self.cache.values()
            .map(|e| e.embedding.len() * std::mem::size_of::<f32>())
            .sum::<usize>();
        
        (total_embeddings, total_memory)
    }

    /// Clear embedding cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        info!("Cleared embedding cache");
    }

    /// Generate mock embedding (placeholder implementation)
    async fn generate_mock_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Simple hash-based mock embedding for testing
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Generate deterministic but varied embedding
        let mut embedding = Vec::with_capacity(self.config.dimension);
        let mut seed = hash;
        
        for _ in 0..self.config.dimension {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let value = ((seed >> 16) as f32) / 32768.0 - 1.0; // Range [-1, 1]
            embedding.push(value);
        }
        
        // Normalize the vector
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for value in &mut embedding {
                *value /= magnitude;
            }
        }
        
        Ok(embedding)
    }

    /// Find similar embeddings using cosine similarity
    pub fn find_similar(&self, query_embedding: &[f32], threshold: f32, limit: usize) -> Vec<(String, f32)> {
        let mut similarities: Vec<(String, f32)> = self.cache
            .iter()
            .map(|(id, embedding)| {
                let similarity = cosine_similarity(query_embedding, &embedding.embedding);
                (id.clone(), similarity)
            })
            .filter(|(_, similarity)| *similarity >= threshold)
            .collect();
        
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similarities.truncate(limit);
        
        similarities
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (magnitude_a * magnitude_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_generation() {
        let embeddings = Embeddings::new().unwrap();
        let result = embeddings.embed("Hello world", "test-model").await;
        assert!(result.is_ok());
        
        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 384); // Default dimension
    }

    #[test]
    fn test_text_chunking() {
        let mut embeddings = Embeddings::new().unwrap();
        embeddings.config.chunk_size = 50;
        embeddings.config.overlap_size = 10;
        
        let text = "This is a long text that should be split into multiple chunks for processing";
        let chunks = embeddings.chunk_text(text);
        
        assert!(chunks.len() > 1);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let similarity = cosine_similarity(&a, &b);
        assert!((similarity - 1.0).abs() < 1e-6);
        
        let c = vec![0.0, 1.0, 0.0];
        let similarity = cosine_similarity(&a, &c);
        assert!((similarity - 0.0).abs() < 1e-6);
    }
}