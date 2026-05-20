use anyhow::Result;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Signature {
    pub payload: String,
    pub hash: String,
    pub timestamp: u64,
    pub actor: String,
}

pub fn sign_action(action: &str, actor: &str) -> Result<String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let payload = format!("{}:{}:{}", actor, timestamp, action);
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    let signature = format!("{}.{}.{}", hash, timestamp, actor);
    Ok(signature)
}

pub fn verify_signature(signature: &str, action: &str) -> Result<bool> {
    let parts: Vec<&str> = signature.split('.').collect();
    if parts.len() != 3 {
        return Ok(false);
    }

    let hash = parts[0];
    let timestamp = parts[1];
    let actor = parts[2];

    let payload = format!("{}:{}:{}", actor, timestamp, action);
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let expected_hash = format!("{:x}", hasher.finalize());

    Ok(hash == expected_hash)
}

pub fn generate_keypair() -> Result<(String, String)> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let private: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    let public: Vec<u8> = (0..32).map(|_| rng.gen()).collect();

    Ok((
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, private),
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, public),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let sig = sign_action("deploy: v1.2.3", "admin").unwrap();
        let valid = verify_signature(&sig, "deploy: v1.2.3").unwrap();
        assert!(valid);
    }

    #[test]
    fn test_signature_tampered() {
        let sig = sign_action("deploy: v1.2.3", "admin").unwrap();
        let valid = verify_signature(&sig, "deploy: v1.2.4").unwrap();
        assert!(!valid);
    }
}
