# GitHub PR Review MCP Server - Zed Extension Migration Plan

## Executive Summary

This document outlines the comprehensive plan for adapting the `zed-pal-mcp-server` codebase to create a Zed extension for the `github-pr-review-mcp-server`. The migration leverages the proven architecture and patterns established in the PAL extension while adapting them to the specific requirements of the GitHub PR review tooling.

**Source Repository:** https://github.com/petems/zed-pal-mcp-server
**Target MCP Server:** https://github.com/petems/github-pr-review-mcp-server
**New Extension Name:** `zed-github-pr-review-mcp-server`

---

## 1. High-Level Architecture Overview

### Current PAL Extension Architecture

The PAL extension follows a clean, modular architecture:

1. **Extension Manifest** (`extension.toml`): Declares extension metadata and context server registration
2. **Rust Extension Logic** (`src/lib.rs`): Implements the `zed::Extension` trait with two key methods:
   - `context_server_command()`: Returns the command to launch the MCP server
   - `context_server_configuration()`: Returns UI configuration (settings schema, defaults, instructions)
3. **Settings Structure**: Uses `serde` and `schemars` to define a Rust struct that auto-generates JSON schema for Zed's settings UI
4. **Command Resolution**: Implements a fallback chain (uvx → uv → python) to maximize compatibility
5. **Environment Variables**: Maps Zed settings to environment variables consumed by the Python MCP server

### GitHub PR Review MCP Server Architecture

The target server is a Python-based MCP server with:

- **Package Name**: `mcp-github-pr-review`
- **Executable Name**: `mcp-github-pr-review`
- **Runtime**: Python 3.10+ with `uv` support
- **Primary Configuration**: Environment variables (especially `GITHUB_TOKEN`)
- **Key Tools**:
  - `fetch_pr_review_comments`: Fetches and formats PR review comments
  - `resolve_open_pr_url`: Auto-resolves PR URL from git context
  - `resolve_pr_review_thread`: Marks review threads as resolved

### Migration Strategy

The migration will **preserve the proven architecture** while adapting:
1. **Identifiers**: Replace `pal-mcp-server` with `github-pr-review-mcp-server`
2. **Settings**: Replace PAL's 40+ AI provider settings with GitHub-specific configuration
3. **Command Resolution**: Keep the same uvx/uv/python fallback chain (server uses same tooling)
4. **Documentation**: Update installation instructions and examples for PR review use cases

---

## 2. Detailed File-by-File Changes

### 2.1 Extension Manifest (`extension.toml`)

**Current (PAL):**
```toml
id = "pal-mcp-server"
name = "PAL MCP Server"
description = "Zed extension for the PAL MCP server"
version = "0.1.0"
schema_version = 1
repository = "https://github.com/petems/zed-pal-mcp-server"

[context_servers.pal-mcp-server]
name = "PAL MCP Server"
```

**Changes Required:**
```toml
id = "github-pr-review-mcp-server"
name = "GitHub PR Review MCP Server"
description = "Zed extension for fetching and managing GitHub PR review comments"
version = "0.1.0"
schema_version = 1
repository = "https://github.com/petems/zed-github-pr-review-mcp-server"

[context_servers.github-pr-review-mcp-server]
name = "GitHub PR Review MCP Server"
```

**Impact:** Low complexity - simple string replacements

---

### 2.2 Rust Extension Logic (`src/lib.rs`)

This is the main implementation file requiring significant changes.

#### 2.2.1 Constants Section (Lines 11-22)

**Current (PAL):**
```rust
const CONTEXT_SERVER_ID: &str = "pal-mcp-server";
const PAL_PACKAGE_NAME: &str = "pal-mcp-server";
const PAL_EXECUTABLE_NAME: &str = "pal-mcp-server";
const PYTHON_MIN_MAJOR: u32 = 3;
const PYTHON_MIN_MINOR: u32 = 10;
const PYTHON_VERSION_SCRIPT: &str =
    "import sys; print(f'{sys.version_info[0]}.{sys.version_info[1]}')";
const PYTHON_SCRIPTS_DIR_SCRIPT: &str = concat!(
    "import os, sysconfig; ",
    "suffix = '.exe' if os.name == 'nt' else ''; ",
    "print(os.path.join(sysconfig.get_path('scripts'), 'pal-mcp-server' + suffix))"
);
```

**Changes Required:**
```rust
const CONTEXT_SERVER_ID: &str = "github-pr-review-mcp-server";
const PACKAGE_NAME: &str = "mcp-github-pr-review";  // From pyproject.toml
const EXECUTABLE_NAME: &str = "mcp-github-pr-review";  // From [project.scripts]
const PYTHON_MIN_MAJOR: u32 = 3;
const PYTHON_MIN_MINOR: u32 = 10;
const PYTHON_VERSION_SCRIPT: &str =
    "import sys; print(f'{sys.version_info[0]}.{sys.version_info[1]}')";
const PYTHON_SCRIPTS_DIR_SCRIPT: &str = concat!(
    "import os, sysconfig; ",
    "suffix = '.exe' if os.name == 'nt' else ''; ",
    "print(os.path.join(sysconfig.get_path('scripts'), 'mcp-github-pr-review' + suffix))"
);
```

**Key Points:**
- Package name from `pyproject.toml`: `mcp-github-pr-review`
- Executable from `[project.scripts]`: `mcp-github-pr-review`
- Keep Python 3.10+ requirement (same as PAL)
- Keep same Python detection scripts (proven pattern)

---

#### 2.2.2 Extension Struct (Line 24)

**Current (PAL):**
```rust
struct PalMcpServerExtension;
```

**Changes Required:**
```rust
struct GitHubPrReviewMcpServerExtension;
```

**Impact:** Must update in two places:
1. Struct definition (line 24)
2. Trait implementation (line 72)
3. Registration macro (line 377)

---

#### 2.2.3 Settings Structure (Lines 26-64)

This requires the most significant changes. PAL has 40+ fields for various AI providers. GitHub PR Review has a much simpler configuration focused on GitHub integration.

**Current (PAL) - Abbreviated:**
```rust
#[derive(Debug, Default, Deserialize, JsonSchema)]
struct PalMcpServerSettings {
    gemini_api_key: Option<String>,
    openai_api_key: Option<String>,
    azure_openai_api_key: Option<String>,
    // ... 35+ more fields for AI providers
    log_level: Option<String>,
    disabled_tools: Option<String>,
}
```

**Changes Required:**

Based on the `.env.example` file from the GitHub PR Review server, the required settings are:

```rust
#[derive(Debug, Default, Deserialize, JsonSchema)]
struct GitHubPrReviewMcpServerSettings {
    // Required: GitHub authentication
    github_token: Option<String>,

    // GitHub Enterprise support (optional)
    gh_host: Option<String>,
    github_api_url: Option<String>,
    github_graphql_url: Option<String>,

    // Safety and performance limits (optional)
    pr_fetch_max_pages: Option<u32>,
    pr_fetch_max_comments: Option<u32>,
    http_per_page: Option<u32>,
    http_max_retries: Option<u32>,

    // General settings (optional)
    log_level: Option<String>,
}
```

**Environment Variable Mapping:**

From `.env.example`:
```bash
GITHUB_TOKEN=your_github_token_here       # REQUIRED
GH_HOST=github.com                        # Optional, defaults to github.com
GITHUB_API_URL=https://api.github.com     # Optional, derived from GH_HOST
GITHUB_GRAPHQL_URL=https://api.github.com/graphql  # Optional, derived from GH_HOST
PR_FETCH_MAX_PAGES=50                     # Optional, default 50
PR_FETCH_MAX_COMMENTS=2000                # Optional, default 2000
HTTP_PER_PAGE=100                         # Optional, default 100
HTTP_MAX_RETRIES=3                        # Optional, default 3
```

**Impact:** Major simplification - from 40+ fields to ~10 fields

---

#### 2.2.4 Settings-to-Environment Conversion (Lines 114-205)

**Current (PAL) - Abbreviated:**
```rust
impl PalMcpServerSettings {
    fn into_env_vars(self) -> Vec<(String, String)> {
        let mut env = Vec::new();
        push_string_env(&mut env, "GEMINI_API_KEY", self.gemini_api_key);
        push_string_env(&mut env, "OPENAI_API_KEY", self.openai_api_key);
        // ... 30+ more push_*_env() calls
        env
    }
}
```

**Changes Required:**
```rust
impl GitHubPrReviewMcpServerSettings {
    fn into_env_vars(self) -> Vec<(String, String)> {
        let mut env = Vec::new();

        // Required: GitHub token
        push_string_env(&mut env, "GITHUB_TOKEN", self.github_token);

        // Optional: GitHub Enterprise configuration
        push_string_env(&mut env, "GH_HOST", self.gh_host);
        push_string_env(&mut env, "GITHUB_API_URL", self.github_api_url);
        push_string_env(&mut env, "GITHUB_GRAPHQL_URL", self.github_graphql_url);

        // Optional: Safety and performance limits
        push_value_env(&mut env, "PR_FETCH_MAX_PAGES", self.pr_fetch_max_pages);
        push_value_env(&mut env, "PR_FETCH_MAX_COMMENTS", self.pr_fetch_max_comments);
        push_value_env(&mut env, "HTTP_PER_PAGE", self.http_per_page);
        push_value_env(&mut env, "HTTP_MAX_RETRIES", self.http_max_retries);

        // Optional: General settings
        push_string_env(&mut env, "LOG_LEVEL", self.log_level);

        env
    }
}
```

**Impact:** Major simplification - much cleaner and easier to maintain

---

#### 2.2.5 Settings Loader Function (Lines 207-212)

**Current (PAL):**
```rust
fn load_pal_settings(settings: Option<serde_json::Value>) -> Result<PalMcpServerSettings> {
    match settings {
        Some(settings) => serde_json::from_value(settings).map_err(|err| err.to_string()),
        None => Ok(PalMcpServerSettings::default()),
    }
}
```

**Changes Required:**
```rust
fn load_settings(settings: Option<serde_json::Value>) -> Result<GitHubPrReviewMcpServerSettings> {
    match settings {
        Some(settings) => serde_json::from_value(settings).map_err(|err| err.to_string()),
        None => Ok(GitHubPrReviewMcpServerSettings::default()),
    }
}
```

**Impact:** Simple rename - logic unchanged

---

#### 2.2.6 Command Override Function (Lines 214-236)

**Current (PAL):**
```rust
fn command_from_override(
    command_settings: &CommandSettings,
    derived_env: Vec<(String, String)>,
) -> Result<Command> {
    let path = command_settings.path.clone().ok_or_else(|| {
        "`context_servers.pal-mcp-server.command.path` is required when overriding the command"
            .to_string()
    })?;
    // ... rest of implementation
}
```

**Changes Required:**
```rust
fn command_from_override(
    command_settings: &CommandSettings,
    derived_env: Vec<(String, String)>,
) -> Result<Command> {
    let path = command_settings.path.clone().ok_or_else(|| {
        "`context_servers.github-pr-review-mcp-server.command.path` is required when overriding the command"
            .to_string()
    })?;
    // ... rest of implementation unchanged
}
```

**Impact:** Minimal - only error message changes

---

#### 2.2.7 Command Resolution Function (Lines 238-257)

**Current (PAL):**
```rust
fn resolve_default_command() -> Result<Command> {
    if command_available("uvx", &["--version"]) {
        return Ok(Command::new("uvx").args(["--from", PAL_PACKAGE_NAME, PAL_EXECUTABLE_NAME]));
    }

    if command_available("uv", &["tool", "run", "--help"]) {
        return Ok(Command::new("uv").args([
            "tool",
            "run",
            "--from",
            PAL_PACKAGE_NAME,
            PAL_EXECUTABLE_NAME,
        ]));
    }

    let python = find_supported_python()?;
    install_pal_with_python(&python)?;
    let pal_executable = python_pal_executable(&python)?;
    Ok(Command::new(pal_executable))
}
```

**Changes Required:**
```rust
fn resolve_default_command() -> Result<Command> {
    if command_available("uvx", &["--version"]) {
        return Ok(Command::new("uvx").args(["--from", PACKAGE_NAME, EXECUTABLE_NAME]));
    }

    if command_available("uv", &["tool", "run", "--help"]) {
        return Ok(Command::new("uv").args([
            "tool",
            "run",
            "--from",
            PACKAGE_NAME,
            EXECUTABLE_NAME,
        ]));
    }

    let python = find_supported_python()?;
    install_with_python(&python)?;
    let executable = python_executable(&python)?;
    Ok(Command::new(executable))
}
```

**Key Points:**
- Same fallback strategy (uvx → uv → python)
- Both servers use the same Python/uv ecosystem
- Only package/executable names change
- Architecture proven and battle-tested

**Impact:** Minimal - just variable name changes

---

#### 2.2.8 Python Helper Functions (Lines 268-363)

**Functions to Update:**

1. `find_supported_python()` (lines 268-294)
   - Change error message from "PAL" to "GitHub PR Review MCP Server"

2. `install_pal_with_python()` → `install_with_python()` (lines 309-332)
   - Rename function
   - Change `PAL_PACKAGE_NAME` to `PACKAGE_NAME`
   - Update error messages

3. `python_pal_executable()` → `python_executable()` (lines 334-351)
   - Rename function
   - Update `PYTHON_SCRIPTS_DIR_SCRIPT` to use new executable name
   - Update error messages

**Example Change:**
```rust
// OLD
fn install_pal_with_python(candidate: &PythonCommand) -> Result<()> {
    let output = run_python(
        candidate,
        ["-m", "pip", "install", "--user", "--upgrade", PAL_PACKAGE_NAME],
    )?;
    // ...
}

// NEW
fn install_with_python(candidate: &PythonCommand) -> Result<()> {
    let output = run_python(
        candidate,
        ["-m", "pip", "install", "--user", "--upgrade", PACKAGE_NAME],
    )?;
    // ...
}
```

**Impact:** Low - mostly mechanical renaming

---

#### 2.2.9 Extension Trait Implementation (Lines 72-112)

**Current (PAL):**
```rust
impl zed::Extension for PalMcpServerExtension {
    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<Command> {
        let server_settings = ContextServerSettings::for_project(CONTEXT_SERVER_ID, project)?;
        let pal_settings = load_pal_settings(server_settings.settings)?;
        let env = pal_settings.into_env_vars();
        // ... rest of implementation
    }
}
```

**Changes Required:**
```rust
impl zed::Extension for GitHubPrReviewMcpServerExtension {
    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<Command> {
        let server_settings = ContextServerSettings::for_project(CONTEXT_SERVER_ID, project)?;
        let settings = load_settings(server_settings.settings)?;
        let env = settings.into_env_vars();
        // ... rest of implementation unchanged
    }

    fn context_server_configuration(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Option<ContextServerConfiguration>> {
        let installation_instructions =
            include_str!("../configuration/installation_instructions.md").to_string();
        let default_settings = include_str!("../configuration/default_settings.jsonc").to_string();
        let settings_schema = serde_json::to_string(&schemars::schema_for!(GitHubPrReviewMcpServerSettings))
            .map_err(|err| err.to_string())?;

        Ok(Some(ContextServerConfiguration {
            installation_instructions,
            default_settings,
            settings_schema,
        }))
    }
}
```

**Impact:** Minimal - type names and function calls updated

---

#### 2.2.10 Extension Registration (Line 377)

**Current (PAL):**
```rust
zed::register_extension!(PalMcpServerExtension);
```

**Changes Required:**
```rust
zed::register_extension!(GitHubPrReviewMcpServerExtension);
```

**Impact:** Trivial - one-line change

---

### 2.3 Configuration Files

#### 2.3.1 Default Settings (`configuration/default_settings.jsonc`)

**Current (PAL):**
```jsonc
{
  // All PAL settings are optional. Add only the providers and options you use.
  "openai_api_key": "",
  "gemini_api_key": "",
  "xai_api_key": "",
  "openrouter_api_key": "",
  "azure_openai_api_key": "",
  "azure_openai_endpoint": "",
  "default_model": "auto",
  "log_level": "INFO",
  "disabled_tools": "analyze,refactor,testgen,secaudit,docgen,tracer"
}
```

**Changes Required:**
```jsonc
{
  // GitHub authentication (required)
  "github_token": "",

  // GitHub Enterprise configuration (optional)
  // For GitHub.com (default), these can be omitted
  // "gh_host": "github.com",
  // "github_api_url": "https://api.github.com",
  // "github_graphql_url": "https://api.github.com/graphql",

  // For GitHub Enterprise Server, uncomment and configure:
  // "gh_host": "github.enterprise.com",
  // "github_api_url": "https://github.enterprise.com/api/v3",
  // "github_graphql_url": "https://github.enterprise.com/api/graphql",

  // Performance and safety limits (optional)
  // "pr_fetch_max_pages": 50,
  // "pr_fetch_max_comments": 2000,
  // "http_per_page": 100,
  // "http_max_retries": 3,

  // Logging (optional)
  "log_level": "INFO"
}
```

**Impact:** Complete rewrite - much simpler configuration

---

#### 2.3.2 Installation Instructions (`configuration/installation_instructions.md`)

**Current (PAL):**
```markdown
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
```

**Changes Required:**
```markdown
# GitHub PR Review MCP Server

This extension allows you to fetch and manage GitHub pull request review comments directly in Zed using Claude or other AI assistants.

## Features

- **Fetch PR review comments**: Retrieve all review comments from a pull request in Markdown or JSON format
- **Auto-resolve PRs**: Automatically detect and resolve PR URLs from your current git branch
- **Mark threads resolved**: Close review threads after addressing feedback
- **GitHub Enterprise support**: Works with both GitHub.com and GitHub Enterprise Server

## Installation

This extension starts the GitHub PR Review MCP server with the first available runtime:

1. `uvx --from mcp-github-pr-review mcp-github-pr-review`
2. `uv tool run --from mcp-github-pr-review mcp-github-pr-review`
3. Python 3.10+ with `pip install --user --upgrade mcp-github-pr-review`

### Recommended Setup

1. **Install uv** (recommended): https://docs.astral.sh/uv/getting-started/installation/
2. **Generate a GitHub token**:
   - Go to https://github.com/settings/tokens
   - For classic tokens: Grant `repo` scope (or `public_repo` for public repositories only)
   - For fine-grained tokens: Grant repository access and "Pull requests: Read and write" permission
3. **Configure in Zed**:
   - Open Zed settings (Cmd+, or Ctrl+,)
   - Add your GitHub token
   - Enable the extension in the Agent panel

### Example Settings

**For GitHub.com:**
```jsonc
{
  "context_servers": {
    "github-pr-review-mcp-server": {
      "settings": {
        "github_token": "ghp_your_token_here",
        "log_level": "INFO"
      }
    }
  }
}
```

**For GitHub Enterprise Server:**
```jsonc
{
  "context_servers": {
    "github-pr-review-mcp-server": {
      "settings": {
        "github_token": "your_enterprise_token",
        "gh_host": "github.enterprise.com",
        "github_api_url": "https://github.enterprise.com/api/v3",
        "github_graphql_url": "https://github.enterprise.com/api/graphql",
        "log_level": "INFO"
      }
    }
  }
}
```

## Usage

Once enabled, you can ask Claude to:

- "Fetch the review comments from this PR"
- "Show me all unresolved review comments"
- "What review feedback is there on PR #123?"
- "Mark this review thread as resolved"

The extension will use your current git context to automatically detect the relevant PR when possible.

## Security

- **Token Safety**: Your GitHub token is passed securely via environment variables and never logged
- **Read-only by default**: Most operations only require read access to pull requests
- **Fine-grained permissions**: Use GitHub's fine-grained tokens for least-privilege access

For more details, see: https://github.com/petems/github-pr-review-mcp-server/blob/main/SECURITY.md

## Troubleshooting

**Extension not starting:**
- Ensure you have Python 3.10+ or `uv` installed
- Check that your GitHub token is set and valid
- Look for error messages in Zed's log panel (View → Toggle Log)

**Can't fetch comments:**
- Verify your token has appropriate permissions (`repo` or `public_repo` scope)
- Ensure the PR URL or repository is accessible with your token
- Check if you're behind a firewall blocking GitHub API access

**GitHub Enterprise not working:**
- Confirm your `gh_host`, `github_api_url`, and `github_graphql_url` settings are correct
- Verify your token is valid for your enterprise instance
- Check network connectivity to your enterprise server
```

**Impact:** Complete rewrite - new features and use cases

---

### 2.4 Build Configuration Files

#### 2.4.1 Cargo.toml

**Current (PAL):**
```toml
[package]
name = "zed-pal-mcp-server"
version = "0.1.0"
edition = "2021"
publish = false

[lib]
crate-type = ["cdylib"]

[dependencies]
zed_extension_api = "0.7.0"
serde = { version = "1.0", features = ["derive"] }
schemars = { version = "1.0", features = ["derive"] }
```

**Changes Required:**
```toml
[package]
name = "zed-github-pr-review-mcp-server"
version = "0.1.0"
edition = "2021"
publish = false

[lib]
crate-type = ["cdylib"]

[dependencies]
zed_extension_api = "0.7.0"
serde = { version = "1.0", features = ["derive"] }
schemars = { version = "1.0", features = ["derive"] }
```

**Impact:** Minimal - only package name changes, dependencies unchanged

---

#### 2.4.2 rust-toolchain.toml

**No changes required** - this file is environment configuration and should remain identical:

```toml
[toolchain]
channel = "stable"
components = ["clippy", "rustfmt"]
targets = ["wasm32-wasip1"]
```

---

### 2.5 CI/CD Workflows

#### 2.5.1 CI Workflow (`.github/workflows/ci.yml`)

**Current (PAL):**
```yaml
name: CI

on:
  pull_request:
    branches: [master]
  push:
    branches: [master]

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: rustup target add wasm32-wasip1
      - run: cargo fmt --check
      - run: cargo check --locked --target wasm32-wasip1
      - run: cargo build --locked --target wasm32-wasip1
```

**Changes Required:**
- No functional changes needed
- Optionally update branch name if different from `master`
- Consider adding step to test Python server availability

**Example Enhancement:**
```yaml
name: CI

on:
  pull_request:
    branches: [main]  # Update if using 'main' instead of 'master'
  push:
    branches: [main]

jobs:
  check:
    name: Check Extension
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: rustup target add wasm32-wasip1
      - run: cargo fmt --check
      - run: cargo check --locked --target wasm32-wasip1
      - run: cargo build --locked --target wasm32-wasip1

  verify-server:
    name: Verify MCP Server
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: astral-sh/setup-uv@v5
      - name: Test server installation
        run: |
          uvx --version
          # Optionally: uvx --from mcp-github-pr-review --help
```

**Impact:** Optional enhancement - basic CI works unchanged

---

#### 2.5.2 Release Workflow (`.github/workflows/release.yml`)

**Current (PAL):**
```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: rustup target add wasm32-wasip1
      - run: cargo build --release --target wasm32-wasip1
      - uses: huacnlee/zed-extension-action@v1
        with:
          github_token: ${{ secrets.COMMITTER_TOKEN }}
```

**Changes Required:**
```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: rustup target add wasm32-wasip1
      - run: cargo build --release --target wasm32-wasip1
      - uses: huacnlee/zed-extension-action@v1
        with:
          github_token: ${{ secrets.COMMITTER_TOKEN }}
          # Note: Ensure COMMITTER_TOKEN has repo and workflow scopes
```

**Important Notes:**
- Requires `COMMITTER_TOKEN` secret configured in repository
- Token needs `repo` and `workflow` scopes
- Action notifies Zed extensions repository automatically

**Impact:** Minimal - just ensure secrets are configured

---

### 2.6 Documentation Files

#### 2.6.1 README.md

**Current (PAL):** Documents PAL-specific setup and features

**Changes Required:**
Complete rewrite covering:

1. **Introduction**: GitHub PR review integration for Zed
2. **Features**:
   - Fetch PR review comments
   - Auto-resolve PR URLs
   - Mark threads as resolved
   - GitHub Enterprise support
3. **Installation**:
   - Prerequisites (uv or Python 3.10+)
   - Extension installation in Zed
   - GitHub token setup
4. **Configuration**:
   - GitHub.com setup
   - GitHub Enterprise setup
   - Token permissions
5. **Usage Examples**:
   - Asking Claude to fetch comments
   - Resolving threads
   - Working with multiple PRs
6. **Development**:
   - Building from source
   - Testing locally
   - Contributing guidelines
7. **Troubleshooting**:
   - Common issues
   - Debug logging
   - Token problems

**Example Structure:**
```markdown
# Zed GitHub PR Review MCP Server Extension

A Zed extension that integrates the GitHub PR Review MCP Server, enabling AI assistants like Claude to fetch and manage GitHub pull request review comments directly within Zed.

## Features

- 🔍 **Fetch PR Review Comments**: Get all review comments from any PR in Markdown or JSON
- 🎯 **Auto-Resolve PRs**: Automatically detect PR URLs from your current git branch
- ✅ **Mark Threads Resolved**: Close review threads after addressing feedback
- 🏢 **GitHub Enterprise**: Full support for GitHub Enterprise Server
- 🔒 **Secure**: Token-based authentication with fine-grained permissions

[Continue with full documentation...]
```

**Impact:** Major rewrite - completely new content

---

#### 2.6.2 .gitignore

**Current (PAL):**
```
/target
```

**Recommended Additions:**
```
/target
*.swp
*.swo
*~
.DS_Store
.vscode/
.idea/
*.wasm
```

**Impact:** Minor enhancement - optional improvements

---

## 3. Testing Strategy

### 3.1 Manual Testing Checklist

After implementing all changes, perform these tests:

#### Environment Setup Tests
- [ ] Extension loads in Zed without errors
- [ ] Settings UI displays correctly
- [ ] Settings schema validates properly
- [ ] Installation instructions render in Zed

#### Command Resolution Tests
- [ ] Test with `uvx` available: `which uvx`
- [ ] Test with only `uv` available: `which uv`
- [ ] Test with only Python 3.10+: `which python3`
- [ ] Test fallback chain by removing tools one by one
- [ ] Verify Python version detection for Python 3.9 (should fail gracefully)

#### Server Integration Tests
- [ ] Server starts successfully with valid GitHub token
- [ ] Server fails gracefully with missing token
- [ ] Server fails gracefully with invalid token
- [ ] Custom command override works
- [ ] Environment variables passed correctly

#### GitHub.com Tests
- [ ] Fetch comments from public repository PR
- [ ] Fetch comments from private repository PR
- [ ] Auto-resolve PR URL from current branch
- [ ] Mark thread as resolved

#### GitHub Enterprise Tests (if applicable)
- [ ] Connect to enterprise instance
- [ ] Fetch comments with enterprise token
- [ ] Verify custom API URLs work
- [ ] Test enterprise-specific features

#### Error Handling Tests
- [ ] Missing GitHub token: shows helpful error
- [ ] Invalid PR URL: shows helpful error
- [ ] Network timeout: retries with backoff
- [ ] Invalid permissions: shows clear message

### 3.2 Automated Testing

**Current State:** PAL extension has no automated tests

**Recommended Additions:**

1. **Unit Tests** (using `cargo test`):
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_settings_to_env_vars() {
           let settings = GitHubPrReviewMcpServerSettings {
               github_token: Some("test_token".to_string()),
               log_level: Some("DEBUG".to_string()),
               ..Default::default()
           };

           let env = settings.into_env_vars();
           assert!(env.contains(&("GITHUB_TOKEN".to_string(), "test_token".to_string())));
           assert!(env.contains(&("LOG_LEVEL".to_string(), "DEBUG".to_string())));
       }

       #[test]
       fn test_empty_settings_filtered() {
           let settings = GitHubPrReviewMcpServerSettings {
               github_token: Some("".to_string()),  // Empty string
               ..Default::default()
           };

           let env = settings.into_env_vars();
           // Empty strings should not be added
           assert!(!env.iter().any(|(k, _)| k == "GITHUB_TOKEN"));
       }
   }
   ```

2. **Integration Tests**:
   - Test with actual Python installation
   - Test command resolution logic
   - Test settings schema generation

3. **CI Enhancements**:
   ```yaml
   - run: cargo test --target wasm32-wasip1
   - run: cargo clippy --target wasm32-wasip1 -- -D warnings
   ```

**Impact:** Significant improvement in reliability and maintainability

---

## 4. Migration Workflow

### 4.1 Step-by-Step Implementation Plan

#### Phase 1: Repository Setup (1-2 hours)
1. Create new repository: `zed-github-pr-review-mcp-server`
2. Copy base structure from `zed-pal-mcp-server`
3. Update `.git/config` and remote URLs
4. Create initial branch: `feature/initial-implementation`

#### Phase 2: Core Changes (3-4 hours)
1. Update `extension.toml` with new identifiers
2. Update `Cargo.toml` package name
3. Modify `src/lib.rs`:
   - Update constants (CONTEXT_SERVER_ID, PACKAGE_NAME, etc.)
   - Replace settings struct
   - Update environment variable mapping
   - Rename functions and types
   - Update error messages
4. Run `cargo check` to verify compilation
5. Run `cargo fmt` to format code
6. Run `cargo build --target wasm32-wasip1`

#### Phase 3: Configuration Updates (2-3 hours)
1. Rewrite `configuration/default_settings.jsonc`
2. Rewrite `configuration/installation_instructions.md`
3. Test settings schema generation
4. Verify JSON schema validates correctly

#### Phase 4: Documentation (2-3 hours)
1. Write new `README.md`
2. Create `CONTRIBUTING.md` (optional)
3. Update or create `LICENSE` file
4. Add examples and screenshots
5. Write troubleshooting guide

#### Phase 5: CI/CD Setup (1-2 hours)
1. Update `.github/workflows/ci.yml`
2. Update `.github/workflows/release.yml`
3. Configure `COMMITTER_TOKEN` secret
4. Test CI pipeline with push
5. Verify release workflow (dry run)

#### Phase 6: Testing (3-4 hours)
1. Install dev extension in Zed: "Install Dev Extension"
2. Run through manual testing checklist
3. Test with various configurations
4. Test error scenarios
5. Verify GitHub integration works end-to-end

#### Phase 7: Release (1-2 hours)
1. Create release notes
2. Tag version: `git tag v0.1.0`
3. Push tag: `git push origin v0.1.0`
4. Monitor release workflow
5. Verify extension appears in Zed extensions registry
6. Test installation from registry

**Total Estimated Time:** 15-20 hours

---

### 4.2 Risk Mitigation

#### High-Risk Areas

1. **Settings Schema Generation**
   - **Risk:** Schema doesn't match Zed's expectations
   - **Mitigation:** Test schema in Zed settings UI before release
   - **Fallback:** Use PAL's working schema as reference

2. **Command Resolution**
   - **Risk:** Server executable not found or fails to start
   - **Mitigation:** Extensive testing with all three fallback methods
   - **Fallback:** Document manual installation steps

3. **Environment Variables**
   - **Risk:** Settings not passed correctly to server
   - **Mitigation:** Add debug logging to verify env vars
   - **Fallback:** Support command override for manual env specification

4. **GitHub Token Security**
   - **Risk:** Token exposed in logs or errors
   - **Mitigation:** Ensure token never appears in error messages
   - **Fallback:** Document token rotation procedure

#### Testing Environments

Recommended test matrix:
- **Operating Systems:** macOS, Linux, Windows
- **Python Versions:** 3.10, 3.11, 3.12
- **Installation Methods:** uvx, uv, pip
- **GitHub Instances:** GitHub.com, GitHub Enterprise (if available)

---

## 5. Key Architectural Decisions

### 5.1 Why Keep the Same Fallback Chain?

**Decision:** Maintain uvx → uv → python fallback chain

**Rationale:**
- Both servers use Python and support `uv`
- Proven pattern that works across environments
- No need to reinvent the wheel
- Users likely have same tooling for both extensions

**Alternative Considered:** Single-method installation (uvx only)
**Rejected Because:** Reduces compatibility, worse user experience

---

### 5.2 Why Simplify Settings Structure?

**Decision:** Reduce from 40+ settings to ~10 settings

**Rationale:**
- GitHub PR Review server has simpler configuration surface
- Fewer settings = easier to use
- Less maintenance burden
- Clearer documentation

**Alternative Considered:** Keep all PAL settings for "future expansion"
**Rejected Because:** YAGNI principle, adds unnecessary complexity

---

### 5.3 Why Not Use Node.js?

**Question:** GitHub PR Review server could theoretically be rewritten in Node.js. Should we use `npx` instead?

**Decision:** Keep Python/uv approach

**Rationale:**
- Server is already Python-based and working
- `uv` is fast and reliable
- Changing server implementation is out of scope
- Python → WASM bridge already proven in PAL extension

**Future Consideration:** If server is rewritten in Node.js, update command resolution to use `npx`

---

### 5.4 Schema Generation Strategy

**Decision:** Auto-generate JSON schema from Rust struct using `schemars`

**Rationale:**
- Type-safe: Schema always matches struct definition
- No manual JSON schema writing
- Zed automatically generates settings UI
- Less error-prone than manual maintenance

**Implementation:**
```rust
let settings_schema = serde_json::to_string(
    &schemars::schema_for!(GitHubPrReviewMcpServerSettings)
).map_err(|err| err.to_string())?;
```

---

## 6. Future Enhancements

### 6.1 Potential Features (Post-MVP)

1. **Offline Mode Cache**
   - Cache PR comments locally
   - Reduce GitHub API calls
   - Work offline with cached data

2. **Multi-PR Support**
   - Track multiple PRs simultaneously
   - Compare review comments across PRs
   - Aggregate statistics

3. **Enhanced Filtering**
   - Filter by review state (pending, approved, changes requested)
   - Filter by reviewer
   - Filter by file or path pattern

4. **Notification Integration**
   - Alert when new comments arrive
   - Track unresolved comment count
   - Show in Zed status bar

5. **Code Navigation**
   - Jump directly to commented lines in Zed
   - Highlight commented code
   - Inline comment display

### 6.2 Technical Debt to Address

1. **Add Automated Tests**
   - Unit tests for settings conversion
   - Integration tests for command resolution
   - Mock tests for GitHub API interactions

2. **Improve Error Messages**
   - More specific guidance for common errors
   - Link to troubleshooting documentation
   - Better token validation feedback

3. **Performance Optimization**
   - Cache GitHub API responses
   - Implement pagination more efficiently
   - Reduce startup time

4. **Documentation**
   - Video walkthrough of installation
   - More usage examples
   - FAQ section

---

## 7. Success Criteria

The migration is considered successful when:

### Functional Requirements
- [ ] Extension loads in Zed without errors
- [ ] Settings UI renders correctly
- [ ] Server starts with uvx/uv/python
- [ ] Can fetch PR comments from GitHub.com
- [ ] Can fetch PR comments from GitHub Enterprise
- [ ] Environment variables passed correctly
- [ ] Error messages are clear and actionable

### Quality Requirements
- [ ] Code compiles with no warnings
- [ ] Passes `cargo fmt --check`
- [ ] Passes `cargo clippy` with no warnings
- [ ] CI pipeline passes on all commits
- [ ] Manual testing checklist 100% complete

### User Experience Requirements
- [ ] Installation takes <5 minutes
- [ ] Configuration is straightforward
- [ ] Documentation is clear and comprehensive
- [ ] Works on macOS, Linux, and Windows

### Release Requirements
- [ ] Extension published to Zed registry
- [ ] Release notes published
- [ ] GitHub repository has clear README
- [ ] License file included

---

## 8. Comparison Summary

### What Stays the Same

✅ **Architecture:** Rust WASM extension calling Python server
✅ **Command Resolution:** uvx → uv → python fallback chain
✅ **Settings System:** Rust struct → JSON schema → Zed UI
✅ **Environment Variables:** Settings mapped to env vars
✅ **Error Handling:** Graceful fallbacks and clear messages
✅ **Build System:** Cargo with wasm32-wasip1 target
✅ **CI/CD:** GitHub Actions with same basic structure

### What Changes

🔄 **Identifiers:** All IDs changed to `github-pr-review-mcp-server`
🔄 **Package Names:** `mcp-github-pr-review` (Python package)
🔄 **Settings:** From 40+ AI provider settings to ~10 GitHub settings
🔄 **Environment Variables:** GitHub-specific (GITHUB_TOKEN, etc.)
🔄 **Documentation:** Complete rewrite for PR review use case
🔄 **Configuration:** Focus on GitHub authentication and enterprise support

### Complexity Comparison

| Aspect | PAL Extension | GitHub PR Review | Change |
|--------|---------------|------------------|--------|
| Settings Fields | 40+ | ~10 | -75% |
| Primary Config | Multiple API keys | Single GitHub token | Simpler |
| Use Case | General AI tooling | Specific PR reviews | Focused |
| Server Type | Python | Python | Same |
| Build Process | Rust → WASM | Rust → WASM | Same |
| Lines of Code | ~378 | ~250 (estimated) | -34% |

---

## 9. Conclusion

The migration from PAL MCP Server to GitHub PR Review MCP Server is straightforward because:

1. **Same Foundation:** Both use Python servers launched by Rust WASM extensions
2. **Proven Patterns:** PAL's architecture is battle-tested and well-designed
3. **Simpler Configuration:** GitHub PR Review has fewer settings, making it easier
4. **Clear Boundaries:** Well-defined separation of concerns (Rust extension vs. Python server)

The main work involves:
- **Mechanical Changes:** Renaming identifiers and updating constants
- **Settings Simplification:** Replacing PAL's complex AI provider settings with GitHub-specific config
- **Documentation:** Writing new docs for PR review use cases

**Estimated Effort:** 15-20 hours for a complete, tested, documented implementation

**Key Success Factor:** Leverage PAL's architecture rather than reinventing it

---

## 10. Appendix

### A. Reference Links

- **PAL MCP Server:** https://github.com/petems/zed-pal-mcp-server
- **GitHub PR Review Server:** https://github.com/petems/github-pr-review-mcp-server
- **Zed Extension API:** https://github.com/zed-industries/zed/tree/main/crates/extension_api
- **MCP Specification:** https://modelcontextprotocol.io/
- **uv Documentation:** https://docs.astral.sh/uv/

### B. Environment Variable Reference

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `GITHUB_TOKEN` | Yes | None | GitHub personal access token |
| `GH_HOST` | No | `github.com` | GitHub instance hostname |
| `GITHUB_API_URL` | No | Derived | REST API base URL |
| `GITHUB_GRAPHQL_URL` | No | Derived | GraphQL API endpoint |
| `PR_FETCH_MAX_PAGES` | No | `50` | Max pagination pages |
| `PR_FETCH_MAX_COMMENTS` | No | `2000` | Max total comments |
| `HTTP_PER_PAGE` | No | `100` | Items per page (1-100) |
| `HTTP_MAX_RETRIES` | No | `3` | Max retries for errors |
| `LOG_LEVEL` | No | `INFO` | Logging level |

### C. GitHub Token Permissions

#### Classic Personal Access Token
- **Public Repos (Read Only):** `public_repo`
- **Private Repos (Read Only):** `repo` (read-only operations still work)
- **Write Operations:** `repo` (full repository access)

#### Fine-Grained Personal Access Token
- **Repository Access:** Select specific repositories
- **Permissions:**
  - Pull requests: **Read** (for fetching comments)
  - Pull requests: **Read and write** (for resolving threads)

### D. File Checklist

Files to create/modify:

- [ ] `extension.toml`
- [ ] `Cargo.toml`
- [ ] `src/lib.rs`
- [ ] `configuration/default_settings.jsonc`
- [ ] `configuration/installation_instructions.md`
- [ ] `.github/workflows/ci.yml`
- [ ] `.github/workflows/release.yml`
- [ ] `README.md`
- [ ] `.gitignore`
- [ ] `LICENSE`

Files that stay the same:

- [x] `rust-toolchain.toml`
- [x] `.github/workflows/` (structure)

---

**Document Version:** 1.0
**Last Updated:** 2026-03-14
**Author:** Migration Analysis
**Status:** Ready for Implementation
