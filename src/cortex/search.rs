use crate::config::Config;
use anyhow::Result;
use std::collections::HashMap;

pub struct SearchEngine {
    pub index: HashMap<String, Vec<String>>,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<String>> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        for (path, content) in &self.index {
            let content_lower = content.to_lowercase();
            let mut score = 0;

            for term in &query_terms {
                if content_lower.contains(*term) {
                    score += 1;
                }
            }

            if score > 0 {
                results.push(format!("{} (score: {})", path, score));
            }
        }

        results.sort();
        Ok(results)
    }

    pub fn add_document(&mut self, path: String, content: String) {
        self.index.insert(path, content);
    }

    pub fn remove_document(&mut self, path: &str) {
        self.index.remove(path);
    }
}

pub async fn semantic_search(config: &Config, query: &str) -> Result<Vec<String>> {
    let project_root = std::env::current_dir()?;
    let mut engine = SearchEngine::new();

    // Index project files
    if let Ok(entries) = std::fs::read_dir(&project_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("")).to_string_lossy().to_string();
                if !name.starts_with('.') {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        engine.add_document(
                            path.strip_prefix(&project_root)
                                .unwrap_or(&path)
                                .to_string_lossy()
                                .to_string(),
                            content,
                        );
                    }
                }
            }
        }
    }

    let results = engine.search(query).await?;

    if results.is_empty() {
        println!("No results found for: '{}'", query);
    } else {
        println!("Search results for '{}':", query);
        for result in results.iter().take(20) {
            println!("  {}", result);
        }
    }

    Ok(results)
}

pub async fn search(query: &str) -> Result<String> {
    let engine = SearchEngine::new();
    let results = engine.search(query).await?;
    Ok(results.join("\n"))
}
