pub mod embeddings;
pub mod indexer;
pub mod knowledge_graph;
pub mod search;

use anyhow::Result;
use std::path::PathBuf;

pub struct Cortex {
    pub project_root: PathBuf,
    pub indexer: indexer::CodeIndexer,
    pub search_engine: search::SearchEngine,
}

impl Cortex {
    pub fn new(project_root: PathBuf) -> Result<Self> {
        let indexer = indexer::CodeIndexer::new(project_root.clone())?;
        let search_engine = search::SearchEngine::new();

        Ok(Self {
            project_root,
            indexer,
            search_engine,
        })
    }

    pub async fn index_project(&mut self) -> Result<usize> {
        self.indexer.index_all().await
    }

    pub async fn search(&self, query: &str) -> Result<String> {
        let results = self.search_engine.search(query).await?;
        Ok(results.join("\n"))
    }
}
