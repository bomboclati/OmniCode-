pub mod ledger;
pub mod signing;

use crate::config::Config;
use anyhow::Result;

pub async fn verify_compliance(config: &Config) -> Result<ComplianceReport> {
    let ledger = ledger::ComplianceLedger::new()?;
    let is_valid = ledger.verify_chain()?;
    let entries = ledger.get_recent_entries(100)?;

    Ok(ComplianceReport {
        compliant: is_valid,
        total_entries: entries.len(),
        last_entry: entries.first().cloned(),
        issues: if is_valid { Vec::new() } else { vec!["Chain verification failed".to_string()] },
    })
}

pub async fn export_audit_report(config: &Config, format: &str) -> Result<String> {
    let ledger = ledger::ComplianceLedger::new()?;
    let entries = ledger.get_all_entries()?;

    match format {
        "json" => Ok(serde_json::to_string_pretty(&entries)?),
        "csv" => {
            let mut csv = String::from("timestamp,action,actor,signature,previous_hash\n");
            for entry in &entries {
                csv.push_str(&format!(
                    "{},{},{},{},{}\n",
                    entry.timestamp, entry.action, entry.actor, entry.signature, entry.previous_hash
                ));
            }
            Ok(csv)
        }
        _ => Ok(serde_json::to_string_pretty(&entries)?),
    }
}

pub fn sign_action(action: &str, actor: &str) -> Result<String> {
    signing::sign_action(action, actor)
}

#[derive(Debug, Clone)]
pub struct ComplianceReport {
    pub compliant: bool,
    pub total_entries: usize,
    pub last_entry: Option<ledger::LedgerEntry>,
    pub issues: Vec<String>,
}
