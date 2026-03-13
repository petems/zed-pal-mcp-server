# Zed PAL MCP Server Extension

## Context

PAL (Provider Abstraction Layer) is a Python-based MCP server that provides multi-model AI orchestration - letting your primary AI tool consult multiple AI models (OpenAI, Gemini, Grok, etc.) through a single MCP interface. This extension wraps PAL as a Zed extension so it's installable from the Zed extension marketplace and configurable via Zed's settings.

## Project Structure

```
zed-pal-mcp-server/
  extension.toml          # Extension manifest
  Cargo.toml              # Rust project (cdylib → WASM)
  src/lib.rs              # All extension logic
  .gitignore
  LICENSE
```

## Files to Create

### 1. `extension.toml`
- Declares extension id `pal-mcp-server`, schema_version 1
- Registers `[context_servers.pal-mcp-server]`

### 2. `Cargo.toml`
- `crate-type = ["cdylib"]`
- Dependencies: `zed_extension_api = "0.7.0"`, `serde`, `schemars = "1.0"`

### 3. `src/lib.rs` — Core Logic

**Settings struct** (`PalMcpServerSettings`):
- Optional fields for all PAL env vars: `gemini_api_key`, `openai_api_key`, `xai_api_key`, `openrouter_api_key`, `azure_openai_api_key`, `azure_openai_endpoint`, `default_model`, `disabled_tools`, `log_level`, etc.
- Derives `Deserialize` + `JsonSchema` (for Zed settings panel)

**`context_server_command()`** — returns `zed::Command`:
1. Read user settings via `ContextServerSettings::for_project()`
2. Build env vars by mapping each `Some(value)` field → `SCREAMING_SNAKE_CASE` env var
3. Find PAL executable with fallback chain:
   - **`uvx`** (preferred): `uvx --from pal-mcp-server pal-mcp-server`
   - **`uv`** fallback: `uv run --from pal-mcp-server pal-mcp-server`
   - **Python fallback**: find Python 3.10+, pip install `pal-mcp-server`, run `pal-mcp-server` console script
4. Return `Command { command, args, env }`

**`context_server_configuration()`** — returns installation instructions + JSON schema for settings

### 4. `.gitignore`
- Standard Rust ignores (`/target`, `Cargo.lock` for libs)

## Key Design Decisions

- **`uvx` as primary install method**: PAL recommends it, creates isolated environments, handles Python version management automatically
- **All settings optional**: PAL works with any subset of API keys — don't block startup for missing keys
- **No binary caching**: We rely on system `uvx`/`python`, detection runs only at server startup
- **PyPI package name** (`pal-mcp-server`) not git URL — more reliable and cached

## Verification

1. `rustup target add wasm32-wasip1`
2. `cargo build --target wasm32-wasip1` — confirm it compiles
3. In Zed: Command Palette → "Extensions: Install Dev Extension" → point to project dir
4. Add API keys to Zed `settings.json` under `context_servers.pal-mcp-server.settings`
5. Open Agent Panel — PAL tools (chat, thinkdeep, codereview, etc.) should appear
6. Test by invoking a PAL tool from the agent panel
