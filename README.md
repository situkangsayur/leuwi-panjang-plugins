# Leuwi Panjang Plugins

WASM-based plugin repository for [Leuwi Panjang Terminal](https://github.com/situkangsayur/leuwi-panjang).

## Plugins

| Plugin | Description | Status |
|--------|-------------|--------|
| `plugin-claude` | Claude CLI Integration | Planned |
| `plugin-gemini` | Gemini CLI Integration | Planned |
| `plugin-ollama` | Local AI (Ollama) Integration | Planned |

## Plugin Architecture

Plugins are WASM modules loaded by Leuwi Panjang's wasmtime runtime. They run sandboxed with explicit capability grants.

See [Plugin System Docs](https://github.com/situkangsayur/leuwi-panjang/blob/main/docs/features/02-plugin-system.md) for details.

## License

MIT
