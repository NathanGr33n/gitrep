//! Configuration Management
//!
//! Handles loading and saving application configuration from
//! ~/.config/gitrep/config.yml

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Theme configuration
    #[serde(default)]
    pub theme: ThemeConfig,
    /// Keybinding configuration
    #[serde(default)]
    pub keybindings: KeybindingConfig,
    /// Performance settings
    #[serde(default)]
    pub performance: PerformanceConfig,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Color scheme name
    #[serde(default = "default_theme")]
    pub name: String,
    /// Whether to use Unicode characters
    #[serde(default = "default_true")]
    pub unicode: bool,
}

/// Keybinding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConfig {
    /// Use vim-style keybindings
    #[serde(default = "default_true")]
    pub vim_mode: bool,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum commits to load at once
    #[serde(default = "default_commit_batch_size")]
    pub commit_batch_size: usize,
    /// Enable caching
    #[serde(default = "default_true")]
    pub enable_cache: bool,
}

fn default_theme() -> String {
    "default".to_string()
}

fn default_true() -> bool {
    true
}

fn default_commit_batch_size() -> usize {
    100
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            keybindings: KeybindingConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: default_theme(),
            unicode: true,
        }
    }
}

impl Default for KeybindingConfig {
    fn default() -> Self {
        Self { vim_mode: true }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            commit_batch_size: default_commit_batch_size(),
            enable_cache: true,
        }
    }
}

impl Config {
    /// Get the configuration file path
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("gitrep").join("config.yml"))
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let path = Self::config_path().ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path().ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = serde_yaml::to_string(self)?;
        fs::write(&path, contents)?;
        Ok(())
    }
}
