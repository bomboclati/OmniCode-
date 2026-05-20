use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRecord {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub suggestion: String,
    pub accepted: bool,
    pub rating: u8,
}

pub struct FeedbackEngine {
    pub history: Vec<FeedbackRecord>,
    pub preferences: HashMap<String, f32>,
}

impl FeedbackEngine {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            preferences: HashMap::new(),
        }
    }

    pub fn record_feedback(&mut self, suggestion: &str, accepted: bool, rating: u8) {
        self.history.push(FeedbackRecord {
            timestamp: chrono::Local::now(),
            suggestion: suggestion.to_string(),
            accepted,
            rating,
        });

        let category = categorize_suggestion(suggestion);
        let entry = self.preferences.entry(category).or_insert(0.5);
        if accepted {
            *entry = (*entry + 0.1).min(1.0);
        } else {
            *entry = (*entry - 0.1).max(0.0);
        }
    }

    pub fn get_preference(&self, category: &str) -> f32 {
        self.preferences.get(category).copied().unwrap_or(0.5)
    }

    pub fn should_auto_apply(&self, category: &str) -> bool {
        self.get_preference(category) > 0.8
    }
}

fn categorize_suggestion(suggestion: &str) -> String {
    let lower = suggestion.to_lowercase();
    if lower.contains("refactor") {
        "refactoring".to_string()
    } else if lower.contains("test") {
        "testing".to_string()
    } else if lower.contains("style") || lower.contains("format") {
        "styling".to_string()
    } else if lower.contains("security") {
        "security".to_string()
    } else {
        "general".to_string()
    }
}
