//! Gemini CLI Integration Plugin for Leuwi Panjang
//! Same architecture as Claude plugin but for Google Gemini CLI.

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub enabled: bool,
    pub model: String,
    pub panel_position: String,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            model: "gemini-2.5-pro".into(),
            panel_position: "right".into(),
        }
    }
}

pub struct GeminiPlugin {
    pub config: GeminiConfig,
}

impl GeminiPlugin {
    pub fn new() -> Self {
        Self { config: GeminiConfig::default() }
    }

    pub fn is_available() -> bool {
        std::process::Command::new("gemini")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub async fn send(&self, msg: &str) -> Result<String, String> {
        let output = tokio::process::Command::new("gemini")
            .arg(msg)
            .output()
            .await
            .map_err(|e| format!("Failed: {e}"))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
