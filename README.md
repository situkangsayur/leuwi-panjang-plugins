# Leuwi Panjang Plugins

Plugin repository for [Leuwi Panjang Terminal](https://github.com/situkangsayur/leuwi-panjang).

## Plugins

### plugin-remote — Remote Access Service
Connect to Leuwi Panjang from mobile/laptop via encrypted WireGuard tunnel.
No public IP needed.

- **Zero-config pairing**: QR code or 6-digit code
- **Embedded WireGuard**: no system WireGuard install required
- **Device management**: list, revoke, per-device permissions
- **Audit trail**: all remote actions logged
- **Mobile access**: terminal view/write, AI, file transfer

### plugin-claude — Claude CLI Integration
Integrate Anthropic Claude CLI into the terminal.

- Side panel conversation
- Send terminal selection to Claude
- Permission gate (Claude asks before executing)
- Audit trail of AI actions

### plugin-gemini — Gemini CLI Integration
Google Gemini CLI integration. Same architecture as Claude plugin.

### plugin-ollama — Local AI (Ollama)
Local AI inference via Ollama. No internet, no API keys, full privacy.

- Connects to Ollama HTTP API (localhost:11434)
- Support any model: codellama, qwen3:1.7b, mistral, llama, etc.
- List models, generate responses
- Zero latency (local inference)

Example with qwen3:
```bash
ollama pull qwen3:1.7b
# Plugin auto-detects available models
```

## Architecture

Each plugin is a Rust crate that interfaces with Leuwi Panjang Terminal.

```
leuwi-panjang-plugins/
├── plugins/
│   ├── plugin-remote/   # WireGuard remote access
│   ├── plugin-claude/   # Claude CLI
│   ├── plugin-gemini/   # Gemini CLI
│   └── plugin-ollama/   # Local AI via Ollama
└── Cargo.toml
```

## License

GPL-3.0 — contributions shared back to community
