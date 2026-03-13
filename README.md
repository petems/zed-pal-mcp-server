# PAL MCP Server for Zed

`pal-mcp-server` is a Zed MCP extension that launches the Python-based
[PAL MCP Server](https://github.com/BeehiveInnovations/pal-mcp-server) through
Zed's Rust extension runtime.

## What It Does

- Exposes PAL as a Zed context server.
- Maps Zed settings to PAL environment variables.
- Starts PAL with the first available runtime:
  - `uvx --from pal-mcp-server pal-mcp-server`
  - `uv tool run --from pal-mcp-server pal-mcp-server`
  - Python 3.10+ with `pip install --user --upgrade pal-mcp-server`

## Install In Zed

1. Open Zed.
2. Run `Extensions: Install Dev Extension`.
3. Select this repository.
4. Configure `context_servers.pal-mcp-server.settings` in your Zed settings.

Example:

```jsonc
{
  "context_servers": {
    "pal-mcp-server": {
      "settings": {
        "openai_api_key": "sk-...",
        "default_model": "auto",
        "log_level": "INFO"
      }
    }
  }
}
```

## Development

Requirements:

- Rust stable
- `wasm32-wasip1` target
- `rustfmt`
- `clippy`
- One of:
  - `uvx`
  - `uv`
  - Python 3.10+ with `pip`

Setup:

```bash
rustup show
rustup target add wasm32-wasip1
cargo fmt
cargo check
cargo build --target wasm32-wasip1
```

## Release Automation

This repo includes:

- `.github/workflows/ci.yml` for Rust formatting and build verification
- `.github/workflows/release.yml` to notify the Zed extensions repository after a tagged release

The release workflow uses
[`huacnlee/zed-extension-action`](https://github.com/huacnlee/zed-extension-action).
Create a `COMMITTER_TOKEN` repository secret with `repo` and `workflow` scopes
so the action can open the update PR against `zed-industries/extensions`.
