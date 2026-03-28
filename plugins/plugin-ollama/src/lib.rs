//! Ollama Local AI Plugin for Leuwi Panjang
//!
//! Runs AI inference locally — no internet, no API keys, full privacy.
//! Connects to Ollama HTTP API (default localhost:11434).

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub enabled: bool,
    pub host: String,
    pub default_model: String,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "http://localhost:11434".into(),
            default_model: "codellama:13b".into(),
        }
    }
}

#[derive(Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

pub struct OllamaPlugin {
    pub config: OllamaConfig,
    client: reqwest::Client,
}

impl OllamaPlugin {
    pub fn new() -> Self {
        Self {
            config: OllamaConfig::default(),
            client: reqwest::Client::new(),
        }
    }

    /// Check if Ollama is running
    pub async fn is_available(&self) -> bool {
        self.client.get(&format!("{}/api/tags", self.config.host))
            .send().await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// List available models
    pub async fn list_models(&self) -> Vec<String> {
        #[derive(Deserialize)]
        struct TagsResponse { models: Vec<ModelInfo> }
        #[derive(Deserialize)]
        struct ModelInfo { name: String }

        match self.client.get(&format!("{}/api/tags", self.config.host))
            .send().await
        {
            Ok(resp) => {
                resp.json::<TagsResponse>().await
                    .map(|t| t.models.into_iter().map(|m| m.name).collect())
                    .unwrap_or_default()
            }
            Err(_) => Vec::new(),
        }
    }

    /// Send a prompt to Ollama
    pub async fn generate(&self, prompt: &str) -> Result<String, String> {
        let req = GenerateRequest {
            model: self.config.default_model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let resp = self.client.post(&format!("{}/api/generate", self.config.host))
            .json(&req)
            .send().await
            .map_err(|e| format!("Ollama error: {e}"))?;

        let gen: GenerateResponse = resp.json().await
            .map_err(|e| format!("Parse error: {e}"))?;

        Ok(gen.response)
    }
}
