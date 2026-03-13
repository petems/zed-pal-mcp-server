# PAL MCP Server

This extension starts the PAL MCP server with the first available runtime:

1. `uvx --from pal-mcp-server pal-mcp-server`
2. `uv tool run --from pal-mcp-server pal-mcp-server`
3. Python 3.10+ with `pip install --user --upgrade pal-mcp-server`

Recommended setup:

1. Install `uv`: https://docs.astral.sh/uv/getting-started/installation/
2. Add at least one provider API key in Zed settings.
3. Open the Agent panel and enable `pal-mcp-server`.

Example settings:

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
