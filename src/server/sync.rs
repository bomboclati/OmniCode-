use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOperation {
    pub id: String,
    pub operation: String,
    pub path: String,
    pub content: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
}

pub struct SyncEngine {
    pub operations: Vec<SyncOperation>,
    pub file_states: HashMap<String, String>,
    pub version: u64,
}

impl SyncEngine {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            file_states: HashMap::new(),
            version: 0,
        }
    }

    pub fn apply_operation(&mut self, op: SyncOperation) {
        self.operations.push(op.clone());
        if let Some(content) = &op.content {
            self.file_states.insert(op.path, content.clone());
        }
        self.version += 1;
    }

    pub fn get_file_state(&self, path: &str) -> Option<&String> {
        self.file_states.get(path)
    }

    pub fn get_operations_since(&self, version: u64) -> Vec<&SyncOperation> {
        self.operations
            .iter()
            .skip(version as usize)
            .collect()
    }
}

pub async fn sync_files(
    engine: &Arc<RwLock<SyncEngine>>,
    operations: Vec<SyncOperation>,
) -> u64 {
    let mut eng = engine.write().await;
    for op in operations {
        eng.apply_operation(op);
    }
    eng.version
}

pub async fn get_sync_state(
    engine: &Arc<RwLock<SyncEngine>>,
) -> (u64, HashMap<String, String>) {
    let eng = engine.read().await;
    (eng.version, eng.file_states.clone())
}
