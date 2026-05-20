use crate::crypto::{decrypt_string, derive_machine_key, encrypt_string, CryptoError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file")]
    ReadFailed(#[from] std::io::Error),
    #[error("Failed to parse config")]
    ParseFailed(#[from] toml::de::Error),
    #[error("Failed to serialize config")]
    SerializeFailed(#[from] toml::ser::Error),
    #[error("Crypto error")]
    CryptoError(#[from] CryptoError),
    #[error("No active profile configured")]
    NoActiveProfile,
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Provider {
    OpenAI,
    Anthropic,
    Azure,
    Groq,
    Together,
    Ollama,
    LMStudio,
    Custom,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::OpenAI => write!(f, "OpenAI"),
            Provider::Anthropic => write!(f, "Anthropic"),
            Provider::Azure => write!(f, "Azure"),
            Provider::Groq => write!(f, "Groq"),
            Provider::Together => write!(f, "Together"),
            Provider::Ollama => write!(f, "Ollama"),
            Provider::LMStudio => write!(f, "LM Studio"),
            Provider::Custom => write!(f, "Custom"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxMode {
    Local,
    Docker,
    Firecracker,
}

impl Default for SandboxMode {
    fn default() -> Self {
        SandboxMode::Local
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub provider: Provider,
    pub api_key: String,
    pub endpoint: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            provider: Provider::OpenAI,
            api_key: String::new(),
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            model: "gpt-4".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub profiles: HashMap<String, Profile>,
    pub active_profile: String,
    pub sandbox_mode: SandboxMode,
    pub collaboration_port: u16,
    pub web_port: u16,
    pub offline_mode: bool,
    pub compliance_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert("default".to_string(), Profile::default());
        Self {
            profiles,
            active_profile: "default".to_string(),
            sandbox_mode: SandboxMode::Local,
            collaboration_port: 9421,
            web_port: 9420,
            offline_mode: false,
            compliance_mode: false,
        }
    }
}

impl Config {
    pub fn config_dir() -> Result<PathBuf, ConfigError> {
        #[cfg(target_os = "windows")]
        {
            let path = std::env::var("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|_| dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")));
            Ok(path.join("omnicode"))
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".omnicode"))
        }
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().unwrap_or_default().join("config.toml")
    }

    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();
        if !path.exists() {
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let dir = Self::config_dir()?;
        fs::create_dir_all(&dir)?;

        let key = derive_machine_key()?;
        let mut serializable = self.clone();
        for profile in serializable.profiles.values_mut() {
            if !profile.api_key.is_empty() {
                profile.api_key = encrypt_string(&profile.api_key, &key)?;
            }
        }

        let content = toml::to_string_pretty(&serializable)?;
        fs::write(Self::config_path(), content)?;
        Ok(())
    }

    pub fn get_active_profile(&self) -> Result<Profile, ConfigError> {
        let profile = self
            .profiles
            .get(&self.active_profile)
            .ok_or_else(|| ConfigError::ProfileNotFound(self.active_profile.clone()))?;

        let mut profile = profile.clone();
        if !profile.api_key.is_empty() {
            let key = derive_machine_key()?;
            profile.api_key = decrypt_string(&profile.api_key, &key)?;
        }
        Ok(profile)
    }

    pub fn set_profile(&mut self, name: String, profile: Profile) {
        self.profiles.insert(name, profile);
    }

    pub fn set_active_profile(&mut self, name: String) -> Result<(), ConfigError> {
        if self.profiles.contains_key(&name) {
            self.active_profile = name;
            Ok(())
        } else {
            Err(ConfigError::ProfileNotFound(name))
        }
    }
}
