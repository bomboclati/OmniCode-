use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Failed to encrypt data")]
    EncryptionFailed,
    #[error("Failed to decrypt data")]
    DecryptionFailed,
    #[error("Invalid base64 encoding")]
    InvalidBase64,
    #[error("Invalid encrypted data format")]
    InvalidFormat,
    #[error("Failed to derive machine key")]
    KeyDerivationFailed,
}

pub fn encrypt_string(plaintext: &str, key: &[u8; 32]) -> Result<String, CryptoError> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| CryptoError::EncryptionFailed)?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(&combined))
}

pub fn decrypt_string(encrypted: &str, key: &[u8; 32]) -> Result<String, CryptoError> {
    let combined = STANDARD
        .decode(encrypted)
        .map_err(|_| CryptoError::InvalidBase64)?;

    if combined.len() < 13 {
        return Err(CryptoError::InvalidFormat);
    }

    let nonce_bytes = &combined[..12];
    let ciphertext = &combined[12..];

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    String::from_utf8(plaintext).map_err(|_| CryptoError::DecryptionFailed)
}

pub fn derive_machine_key() -> Result<[u8; 32], CryptoError> {
    let mut hasher = Sha256::new();

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::ffi::OsStringExt;
        use windows::Win32::System::Com::CoCreateGuid;
        use windows::Win32::System::SystemInformation::GetComputerNameW;

        let mut name_buf = [0u16; 256];
        let mut name_len = name_buf.len() as u32;
        unsafe {
            let _ = GetComputerNameW(name_buf.as_mut_ptr(), &mut name_len);
        }
        let hostname = String::from_utf16_lossy(&name_buf[..name_len as usize]);
        hasher.update(hostname.as_bytes());

        let mut guid = [0u8; 16];
        unsafe {
            let _ = CoCreateGuid(&mut guid);
        }
        hasher.update(&guid);
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(hostname) = std::fs::read_to_string("/etc/hostname") {
            hasher.update(hostname.as_bytes());
        }
        if let Ok(machine_id) = std::fs::read_to_string("/etc/machine-id") {
            hasher.update(machine_id.as_bytes());
        }
    }

    let salt = b"omnicode-local-key-derivation-salt-2024";
    hasher.update(salt);

    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let plaintext = "Hello, OmniCode!";
        let encrypted = encrypt_string(plaintext, &key).unwrap();
        let decrypted = decrypt_string(&encrypted, &key).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_derive_machine_key() {
        let key = derive_machine_key().unwrap();
        assert_eq!(key.len(), 32);
    }
}
