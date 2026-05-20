use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: String,
    pub timestamp: String,
    pub action: String,
    pub actor: String,
    pub details: String,
    pub signature: String,
    pub previous_hash: String,
    pub hash: String,
}

pub struct ComplianceLedger {
    db_path: PathBuf,
}

impl ComplianceLedger {
    pub fn new() -> Result<Self> {
        let config_dir = crate::config::Config::config_dir()?;
        let db_path = config_dir.join("compliance.db");

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ledger (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                action TEXT NOT NULL,
                actor TEXT NOT NULL,
                details TEXT NOT NULL,
                signature TEXT NOT NULL,
                previous_hash TEXT NOT NULL,
                hash TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Self { db_path })
    }

    pub fn append_entry(
        &self,
        action: &str,
        actor: &str,
        details: &str,
        signature: &str,
    ) -> Result<LedgerEntry> {
        let conn = Connection::open(&self.db_path)?;
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();

        let previous_hash = self.get_last_hash(&conn)?;
        let hash = self.compute_hash(&id, &timestamp, action, actor, details, signature, &previous_hash);

        conn.execute(
            "INSERT INTO ledger (id, timestamp, action, actor, details, signature, previous_hash, hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, timestamp, action, actor, details, signature, previous_hash, hash],
        )?;

        Ok(LedgerEntry {
            id,
            timestamp,
            action: action.to_string(),
            actor: actor.to_string(),
            details: details.to_string(),
            signature: signature.to_string(),
            previous_hash,
            hash,
        })
    }

    pub fn verify_chain(&self) -> Result<bool> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, action, actor, details, signature, previous_hash, hash
             FROM ledger ORDER BY rowid ASC",
        )?;

        let entries: Vec<LedgerEntry> = stmt
            .query_map([], |row| {
                Ok(LedgerEntry {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    action: row.get(2)?,
                    actor: row.get(3)?,
                    details: row.get(4)?,
                    signature: row.get(5)?,
                    previous_hash: row.get(6)?,
                    hash: row.get(7)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        let mut expected_previous = String::from("0");

        for entry in &entries {
            if entry.previous_hash != expected_previous {
                return Ok(false);
            }

            let expected = self.compute_hash(
                &entry.id,
                &entry.timestamp,
                &entry.action,
                &entry.actor,
                &entry.details,
                &entry.signature,
                &entry.previous_hash,
            );

            if entry.hash != expected {
                return Ok(false);
            }

            expected_previous = entry.hash.clone();
        }

        Ok(true)
    }

    pub fn get_recent_entries(&self, limit: usize) -> Result<Vec<LedgerEntry>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, action, actor, details, signature, previous_hash, hash
             FROM ledger ORDER BY rowid DESC LIMIT ?1",
        )?;

        let entries = stmt
            .query_map(params![limit as i64], |row| {
                Ok(LedgerEntry {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    action: row.get(2)?,
                    actor: row.get(3)?,
                    details: row.get(4)?,
                    signature: row.get(5)?,
                    previous_hash: row.get(6)?,
                    hash: row.get(7)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }

    pub fn get_all_entries(&self) -> Result<Vec<LedgerEntry>> {
        self.get_recent_entries(usize::MAX)
    }

    fn get_last_hash(&self, conn: &Connection) -> Result<String> {
        let result: Result<String, _> = conn.query_row(
            "SELECT hash FROM ledger ORDER BY rowid DESC LIMIT 1",
            [],
            |row| row.get(0),
        );
        Ok(result.unwrap_or_else(|_| "0".to_string()))
    }

    fn compute_hash(
        &self,
        id: &str,
        timestamp: &str,
        action: &str,
        actor: &str,
        details: &str,
        signature: &str,
        previous_hash: &str,
    ) -> String {
        let payload = format!("{}:{}:{}:{}:{}:{}:{}", id, timestamp, action, actor, details, signature, previous_hash);
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_chain() {
        let ledger = ComplianceLedger::new().unwrap();
        let sig1 = crate::compliance::signing::sign_action("test 1", "alice").unwrap();
        let sig2 = crate::compliance::signing::sign_action("test 2", "bob").unwrap();

        ledger.append_entry("test 1", "alice", "first entry", &sig1).unwrap();
        ledger.append_entry("test 2", "bob", "second entry", &sig2).unwrap();

        assert!(ledger.verify_chain().unwrap());
    }
}
