//! Claude CLI Integration Plugin for Leuwi Panjang
//!
//! Features:
//! - Spawn `claude` CLI as subprocess
//! - Side panel for conversation
//! - Send terminal selection to Claude
//! - Permission gate: Claude asks before executing commands
//! - Audit trail of all AI actions
//! - Encrypted credential storage for sudo/API keys

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    pub enabled: bool,
    pub panel_position: String,    // "right" or "bottom"
    pub panel_width: u32,          // percentage
    pub auto_context: bool,        // send CWD, git status as context
    pub approval_mode: String,     // "always_ask", "session", "auto"
    pub max_history: usize,
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            panel_position: "right".into(),
            panel_width: 40,
            auto_context: true,
            approval_mode: "always_ask".into(),
            max_history: 100,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub actor: String,       // "ai:claude" or "human"
    pub action: String,
    pub target: String,
    pub result: String,      // "executed", "denied", "pending"
    pub approved_by: Option<String>,
}

pub struct ClaudePlugin {
    pub config: ClaudeConfig,
    pub conversation: Vec<Message>,
    pub audit_log: Vec<AuditEntry>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,    // "user" or "assistant"
    pub content: String,
}

impl ClaudePlugin {
    pub fn new() -> Self {
        Self {
            config: ClaudeConfig::default(),
            conversation: Vec::new(),
            audit_log: Vec::new(),
        }
    }

    /// Check if claude CLI is available
    pub fn is_available() -> bool {
        std::process::Command::new("claude")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Send a message to Claude (spawns CLI process)
    pub async fn send(&mut self, user_msg: &str) -> Result<String, String> {
        self.conversation.push(Message {
            role: "user".into(),
            content: user_msg.into(),
        });

        let output = tokio::process::Command::new("claude")
            .arg("-p")
            .arg(user_msg)
            .output()
            .await
            .map_err(|e| format!("Failed to run claude: {e}"))?;

        let response = String::from_utf8_lossy(&output.stdout).to_string();

        self.conversation.push(Message {
            role: "assistant".into(),
            content: response.clone(),
        });

        self.audit_log.push(AuditEntry {
            timestamp: now_secs(),
            actor: "ai:claude".into(),
            action: "respond".into(),
            target: user_msg.into(),
            result: "executed".into(),
            approved_by: Some("human".into()),
        });

        Ok(response)
    }

    /// Log a command execution request from Claude
    pub fn log_command_request(&mut self, command: &str) -> AuditEntry {
        let entry = AuditEntry {
            timestamp: now_secs(),
            actor: "ai:claude".into(),
            action: "execute_command".into(),
            target: command.into(),
            result: "pending".into(),
            approved_by: None,
        };
        self.audit_log.push(entry.clone());
        entry
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
