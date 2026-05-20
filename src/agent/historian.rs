use crate::config::Config;
use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: String,
    pub date: String,
    pub title: String,
    pub summary: String,
    pub full_text: String,
    pub pr_links: String,
    pub tags: String,
    pub context: String,
}

pub struct Historian {
    db_path: PathBuf,
}

impl Historian {
    pub fn new() -> Result<Self> {
        let config_dir = crate::config::Config::config_dir()?;
        let db_path = config_dir.join("decisions.db");

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS decisions (
                id TEXT PRIMARY KEY,
                date TEXT NOT NULL,
                title TEXT NOT NULL,
                summary TEXT NOT NULL,
                full_text TEXT NOT NULL,
                pr_links TEXT DEFAULT '',
                tags TEXT DEFAULT '[]',
                context TEXT DEFAULT ''
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS decision_tags (
                tag TEXT PRIMARY KEY,
                count INTEGER DEFAULT 1
            )",
            [],
        )?;

        Ok(Self { db_path })
    }

    pub fn record_decision(
        &self,
        title: &str,
        summary: &str,
        full_text: &str,
        pr_links: &[String],
        tags: &[String],
        context: &str,
    ) -> Result<Decision> {
        let conn = Connection::open(&self.db_path)?;
        let id = uuid::Uuid::new_v4().to_string();
        let date = chrono::Utc::now().to_rfc3339();
        let pr_json = serde_json::to_string(pr_links).unwrap_or_default();
        let tags_json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT INTO decisions (id, date, title, summary, full_text, pr_links, tags, context)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, date, title, summary, full_text, pr_json, tags_json, context],
        )?;

        for tag in tags {
            conn.execute(
                "INSERT INTO decision_tags (tag, count) VALUES (?1, 1)
                 ON CONFLICT(tag) DO UPDATE SET count = count + 1",
                params![tag],
            )?;
        }

        Ok(Decision {
            id,
            date,
            title: title.to_string(),
            summary: summary.to_string(),
            full_text: full_text.to_string(),
            pr_links: pr_json,
            tags: tags_json,
            context: context.to_string(),
        })
    }

    pub fn search_decisions(&self, query: &str) -> Result<Vec<Decision>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT id, date, title, summary, full_text, pr_links, tags, context
             FROM decisions
             WHERE title LIKE ?1 OR summary LIKE ?1 OR full_text LIKE ?1 OR context LIKE ?1
             ORDER BY date DESC
             LIMIT 50",
        )?;

        let results = stmt
            .query_map(params![format!("%{}%", query)], |row| {
                Ok(Decision {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    title: row.get(2)?,
                    summary: row.get(3)?,
                    full_text: row.get(4)?,
                    pr_links: row.get(5)?,
                    tags: row.get(6)?,
                    context: row.get(7)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    pub fn get_all_decisions(&self, limit: usize) -> Result<Vec<Decision>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT id, date, title, summary, full_text, pr_links, tags, context
             FROM decisions ORDER BY date DESC LIMIT ?1",
        )?;

        let results = stmt
            .query_map(params![limit as i64], |row| {
                Ok(Decision {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    title: row.get(2)?,
                    summary: row.get(3)?,
                    full_text: row.get(4)?,
                    pr_links: row.get(5)?,
                    tags: row.get(6)?,
                    context: row.get(7)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    pub fn get_popular_tags(&self, limit: usize) -> Result<Vec<(String, i64)>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT tag, count FROM decision_tags ORDER BY count DESC LIMIT ?1",
        )?;

        let results = stmt
            .query_map(params![limit as i64], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }
}

pub async fn query_decisions(config: &Config, query: &str) -> Result<()> {
    println!("OmniCode Historian - Querying decision database");
    println!("Query: '{}'\n", query);

    let historian = Historian::new()?;
    let results = historian.search_decisions(query)?;

    if results.is_empty() {
        println!("No decisions found matching your query.");
        println!();
        println!("Try different terms like: architecture, deployment, refactor, security");
        println!("Or list all decisions with: omni why all");
        return Ok(());
    }

    println!("Found {} matching decisions:\n", results.len());
    for decision in &results {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("[{}] {}", &decision.date[..10], decision.title);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let tags: Vec<String> = serde_json::from_str(&decision.tags).unwrap_or_default();
        if !tags.is_empty() {
            println!("Tags: {}", tags.join(", "));
        }

        println!("\n{}", decision.summary);
        println!();
    }

    Ok(())
}

pub async fn capture_decision(
    title: &str,
    summary: &str,
    full_text: &str,
    context: &str,
) -> Result<Decision> {
    let historian = Historian::new()?;
    let decision = historian.record_decision(title, summary, full_text, &[], &[], context)?;
    println!("Decision recorded: {}", title);
    Ok(decision)
}

pub async fn auto_capture_from_pr(pr_data: &PrData, llm: &mut crate::agent::llm_client::LlmClient) -> Result<Option<Decision>> {
    let prompt = format!(
        r#"Analyze this PR description and discussion to extract any architectural or design decisions that were made.
If no significant decisions were made, return null.

PR Title: {}
PR Description: {}

If a decision was made, return JSON with:
- "title": decision title
- "summary": one-line summary
- "full_text": detailed description of the decision and rationale"#,
        pr_data.title, pr_data.description
    );

    match llm.chat(&prompt, &[]).await {
        Ok(response) => {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
                if let Some(title) = json["title"].as_str() {
                    let summary = json["summary"].as_str().unwrap_or("");
                    let full_text = json["full_text"].as_str().unwrap_or("");

                    let decision = capture_decision(title, summary, full_text, &pr_data.description).await?;
                    return Ok(Some(decision));
                }
            }
            Ok(None)
        }
        Err(_) => Ok(None),
    }
}

pub struct PrData {
    pub title: String,
    pub description: String,
    pub discussion: Vec<String>,
}
