use anyhow::Result;

pub struct EmbeddingModel {
    pub model_name: String,
    pub dimensions: usize,
}

impl EmbeddingModel {
    pub fn new() -> Self {
        Self {
            model_name: "text-embedding-3-small".to_string(),
            dimensions: 1536,
        }
    }

    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut embedding = vec![0.0f32; self.dimensions];

        for (i, word) in words.iter().take(self.dimensions).enumerate() {
            let hash = simple_hash(word);
            embedding[i] = (hash % 1000) as f32 / 1000.0;
        }

        Ok(embedding)
    }

    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
}

fn simple_hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}
