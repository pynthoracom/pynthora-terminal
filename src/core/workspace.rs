//! Multi-workspace management support (v0.3.0)
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::core::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub api_key: String,
    pub ingest_url: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceManager {
    workspaces: HashMap<String, Workspace>,
    current: Option<String>,
}

impl WorkspaceManager {
    /// Load workspace manager from file
    pub fn load() -> Result<Self> {
        let path = Self::workspace_file_path()?;
        
        if !path.exists() {
            return Ok(Self {
                workspaces: HashMap::new(),
                current: None,
            });
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read workspace file: {}", path.display()))?;

        let manager: WorkspaceManager = toml::de::from_str(&content)
            .with_context(|| "Failed to parse workspace file")?;

        Ok(manager)
    }

    /// Save workspace manager to file
    pub fn save(&self) -> Result<()> {
        let path = Self::workspace_file_path()?;
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string(self)
            .with_context(|| "Failed to serialize workspace manager")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write workspace file: {}", path.display()))?;

        Ok(())
    }

    /// Add a new workspace
    pub fn add_workspace(&mut self, workspace: Workspace) -> Result<()> {
        self.workspaces.insert(workspace.name.clone(), workspace);
        self.save()?;
        Ok(())
    }

    /// Get current workspace
    pub fn get_current(&self) -> Option<&Workspace> {
        self.current.as_ref().and_then(|name| self.workspaces.get(name))
    }

    /// Set current workspace
    pub fn set_current(&mut self, name: &str) -> Result<()> {
        if !self.workspaces.contains_key(name) {
            return Err(anyhow::anyhow!("Workspace '{}' not found", name));
        }
        self.current = Some(name.to_string());
        self.save()?;
        Ok(())
    }

    /// List all workspaces
    pub fn list(&self) -> Vec<&Workspace> {
        self.workspaces.values().collect()
    }

    /// Convert current workspace to Config
    pub fn to_config(&self) -> Result<Config> {
        let workspace = self.get_current()
            .ok_or_else(|| anyhow::anyhow!("No workspace selected"))?;

        Ok(Config {
            api_key: workspace.api_key.clone(),
            ingest_url: workspace.ingest_url.clone(),
            workspace: workspace.name.clone(),
        })
    }

    fn workspace_file_path() -> Result<PathBuf> {
        let mut path = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
        path.push(".pynthora");
        path.push("workspaces.toml");
        Ok(path)
    }
}

