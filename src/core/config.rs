use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use url::Url;
use validator::Validate;

static CACHED_CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Config {
    #[validate(length(min = 16))]
    pub api_key: String,

    #[validate(url)]
    pub ingest_url: String,

    #[validate(length(min = 1))]
    pub workspace: String,
}

impl Config {
    /// Load configuration from file or environment variables
    pub fn load(custom_path: Option<&str>) -> Result<&'static Config> {
        if let Some(config) = CACHED_CONFIG.get() {
            return Ok(config);
        }

        // Try environment variables first
        if let Some(config) = Self::from_env()? {
            CACHED_CONFIG.set(config).map_err(|_| {
                anyhow::anyhow!("Failed to cache config")
            })?;
            return Ok(CACHED_CONFIG.get().unwrap());
        }

        // Try to load from file
        let config_path = Self::resolve_config_path(custom_path)?;
        let config = Self::from_file(&config_path)?;
        
        CACHED_CONFIG.set(config).map_err(|_| {
            anyhow::anyhow!("Failed to cache config")
        })?;

        Ok(CACHED_CONFIG.get().unwrap())
    }

    /// Load config from environment variables
    fn from_env() -> Result<Option<Config>> {
        let api_key = std::env::var("PYNTHORA_API_KEY").ok();
        let workspace = std::env::var("PYNTHORA_WORKSPACE").ok();

        if api_key.is_none() || workspace.is_none() {
            return Ok(None);
        }

        let config = Config {
            api_key: api_key.unwrap(),
            ingest_url: std::env::var("PYNTHORA_INGEST_URL")
                .unwrap_or_else(|_| "https://api.pynthora.network/ingest".to_string()),
            workspace: workspace.unwrap(),
        };

        config.validate()?;
        Ok(Some(config))
    }

    /// Load config from file
    fn from_file(path: &Path) -> Result<Config> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = if path.extension().and_then(|s| s.to_str()) == Some("yaml")
            || path.extension().and_then(|s| s.to_str()) == Some("yml")
        {
            serde_yaml::from_str(&content)
                .with_context(|| "Failed to parse YAML config")?
        } else {
            serde_json::from_str(&content)
                .with_context(|| "Failed to parse JSON config")?
        };

        config.validate()?;
        Ok(config)
    }

    /// Resolve config file path
    fn resolve_config_path(custom_path: Option<&str>) -> Result<PathBuf> {
        let search_paths: Vec<PathBuf> = vec![
            custom_path.map(PathBuf::from),
            Some(std::env::current_dir()?.join(".pynthorarc")),
            Some(std::env::current_dir()?.join(".pynthorarc.json")),
            Some(std::env::current_dir()?.join(".pynthorarc.yaml")),
        ]
        .into_iter()
        .flatten()
        .collect();

        for path in search_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        anyhow::bail!(
            "No configuration found. Run 'pynthora-terminal init' to generate one or set environment variables."
        );
    }

    /// Save config to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = if path.extension().and_then(|s| s.to_str()) == Some("yaml")
            || path.extension().and_then(|s| s.to_str()) == Some("yml")
        {
            serde_yaml::to_string(self).context("Failed to serialize config to YAML")?
        } else {
            serde_json::to_string_pretty(self).context("Failed to serialize config to JSON")?
        };

        fs::write(path, content).with_context(|| format!("Failed to write config to {}", path.display()))?;
        Ok(())
    }

    /// Get default config path
    pub fn default_path() -> PathBuf {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".pynthorarc.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_validation() {
        let config = Config {
            api_key: "test_key_12345678".to_string(),
            ingest_url: "https://api.pynthora.network/ingest".to_string(),
            workspace: "test-workspace".to_string(),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_save_load() {
        let config = Config {
            api_key: "test_key_12345678".to_string(),
            ingest_url: "https://api.pynthora.network/ingest".to_string(),
            workspace: "test-workspace".to_string(),
        };

        let file = NamedTempFile::new().unwrap();
        config.save(file.path()).unwrap();

        let loaded = Config::from_file(file.path()).unwrap();
        assert_eq!(config.api_key, loaded.api_key);
        assert_eq!(config.workspace, loaded.workspace);
    }
}

