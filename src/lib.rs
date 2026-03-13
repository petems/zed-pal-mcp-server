use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::Deserialize;
use zed::process::Command;
use zed::settings::{CommandSettings, ContextServerSettings};
use zed_extension_api::{
    self as zed, serde_json, ContextServerConfiguration, ContextServerId, Project, Result,
};

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

struct PalMcpServerExtension;

#[derive(Debug, Default, Deserialize, JsonSchema)]
struct PalMcpServerSettings {
    gemini_api_key: Option<String>,
    gemini_base_url: Option<String>,
    openai_api_key: Option<String>,
    azure_openai_api_key: Option<String>,
    azure_openai_endpoint: Option<String>,
    azure_openai_api_version: Option<String>,
    azure_openai_allowed_models: Option<String>,
    azure_models_config_path: Option<String>,
    xai_api_key: Option<String>,
    dial_api_key: Option<String>,
    dial_api_host: Option<String>,
    dial_api_version: Option<String>,
    openrouter_api_key: Option<String>,
    custom_api_url: Option<String>,
    custom_api_key: Option<String>,
    custom_model_name: Option<String>,
    custom_connect_timeout: Option<f64>,
    custom_read_timeout: Option<f64>,
    custom_write_timeout: Option<f64>,
    custom_pool_timeout: Option<f64>,
    default_model: Option<String>,
    default_thinking_mode_thinkdeep: Option<String>,
    openai_allowed_models: Option<String>,
    google_allowed_models: Option<String>,
    xai_allowed_models: Option<String>,
    dial_allowed_models: Option<String>,
    custom_models_config_path: Option<String>,
    conversation_timeout_hours: Option<u32>,
    max_conversation_turns: Option<u32>,
    log_level: Option<String>,
    disabled_tools: Option<String>,
    locale: Option<String>,
    pal_mcp_force_env_override: Option<bool>,
    compose_project_name: Option<String>,
    tz: Option<String>,
    log_max_size: Option<String>,
}

#[derive(Debug, Clone)]
struct PythonCommand {
    program: String,
    prefix_args: Vec<String>,
}

impl zed::Extension for PalMcpServerExtension {
    fn new() -> Self {
        Self
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<Command> {
        let server_settings = ContextServerSettings::for_project(CONTEXT_SERVER_ID, project)?;
        let pal_settings = load_pal_settings(server_settings.settings)?;
        let env = pal_settings.into_env_vars();

        if let Some(command_settings) = server_settings.command.as_ref() {
            return command_from_override(command_settings, env);
        }

        let mut command = resolve_default_command()?;
        command = command.envs(env);
        Ok(command)
    }

    fn context_server_configuration(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Option<ContextServerConfiguration>> {
        let installation_instructions =
            include_str!("../configuration/installation_instructions.md").to_string();
        let default_settings = include_str!("../configuration/default_settings.jsonc").to_string();
        let settings_schema = serde_json::to_string(&schemars::schema_for!(PalMcpServerSettings))
            .map_err(|err| err.to_string())?;

        Ok(Some(ContextServerConfiguration {
            installation_instructions,
            default_settings,
            settings_schema,
        }))
    }
}

impl PalMcpServerSettings {
    fn into_env_vars(self) -> Vec<(String, String)> {
        let mut env = Vec::new();

        push_string_env(&mut env, "GEMINI_API_KEY", self.gemini_api_key);
        push_string_env(&mut env, "GEMINI_BASE_URL", self.gemini_base_url);
        push_string_env(&mut env, "OPENAI_API_KEY", self.openai_api_key);
        push_string_env(&mut env, "AZURE_OPENAI_API_KEY", self.azure_openai_api_key);
        push_string_env(
            &mut env,
            "AZURE_OPENAI_ENDPOINT",
            self.azure_openai_endpoint,
        );
        push_string_env(
            &mut env,
            "AZURE_OPENAI_API_VERSION",
            self.azure_openai_api_version,
        );
        push_string_env(
            &mut env,
            "AZURE_OPENAI_ALLOWED_MODELS",
            self.azure_openai_allowed_models,
        );
        push_string_env(
            &mut env,
            "AZURE_MODELS_CONFIG_PATH",
            self.azure_models_config_path,
        );
        push_string_env(&mut env, "XAI_API_KEY", self.xai_api_key);
        push_string_env(&mut env, "DIAL_API_KEY", self.dial_api_key);
        push_string_env(&mut env, "DIAL_API_HOST", self.dial_api_host);
        push_string_env(&mut env, "DIAL_API_VERSION", self.dial_api_version);
        push_string_env(&mut env, "OPENROUTER_API_KEY", self.openrouter_api_key);
        push_string_env(&mut env, "CUSTOM_API_URL", self.custom_api_url);
        push_string_env(&mut env, "CUSTOM_API_KEY", self.custom_api_key);
        push_string_env(&mut env, "CUSTOM_MODEL_NAME", self.custom_model_name);
        push_value_env(
            &mut env,
            "CUSTOM_CONNECT_TIMEOUT",
            self.custom_connect_timeout,
        );
        push_value_env(&mut env, "CUSTOM_READ_TIMEOUT", self.custom_read_timeout);
        push_value_env(&mut env, "CUSTOM_WRITE_TIMEOUT", self.custom_write_timeout);
        push_value_env(&mut env, "CUSTOM_POOL_TIMEOUT", self.custom_pool_timeout);
        push_string_env(&mut env, "DEFAULT_MODEL", self.default_model);
        push_string_env(
            &mut env,
            "DEFAULT_THINKING_MODE_THINKDEEP",
            self.default_thinking_mode_thinkdeep,
        );
        push_string_env(
            &mut env,
            "OPENAI_ALLOWED_MODELS",
            self.openai_allowed_models,
        );
        push_string_env(
            &mut env,
            "GOOGLE_ALLOWED_MODELS",
            self.google_allowed_models,
        );
        push_string_env(&mut env, "XAI_ALLOWED_MODELS", self.xai_allowed_models);
        push_string_env(&mut env, "DIAL_ALLOWED_MODELS", self.dial_allowed_models);
        push_string_env(
            &mut env,
            "CUSTOM_MODELS_CONFIG_PATH",
            self.custom_models_config_path,
        );
        push_value_env(
            &mut env,
            "CONVERSATION_TIMEOUT_HOURS",
            self.conversation_timeout_hours,
        );
        push_value_env(
            &mut env,
            "MAX_CONVERSATION_TURNS",
            self.max_conversation_turns,
        );
        push_string_env(&mut env, "LOG_LEVEL", self.log_level);
        push_string_env(&mut env, "DISABLED_TOOLS", self.disabled_tools);
        push_string_env(&mut env, "LOCALE", self.locale);
        push_value_env(
            &mut env,
            "PAL_MCP_FORCE_ENV_OVERRIDE",
            self.pal_mcp_force_env_override,
        );
        push_string_env(&mut env, "COMPOSE_PROJECT_NAME", self.compose_project_name);
        push_string_env(&mut env, "TZ", self.tz);
        push_string_env(&mut env, "LOG_MAX_SIZE", self.log_max_size);

        env
    }
}

fn load_pal_settings(settings: Option<serde_json::Value>) -> Result<PalMcpServerSettings> {
    match settings {
        Some(settings) => serde_json::from_value(settings).map_err(|err| err.to_string()),
        None => Ok(PalMcpServerSettings::default()),
    }
}

fn command_from_override(
    command_settings: &CommandSettings,
    derived_env: Vec<(String, String)>,
) -> Result<Command> {
    let path = command_settings.path.clone().ok_or_else(|| {
        "`context_servers.pal-mcp-server.command.path` is required when overriding the command"
            .to_string()
    })?;

    let mut env: BTreeMap<String, String> = derived_env.into_iter().collect();
    if let Some(custom_env) = command_settings.env.as_ref() {
        for (key, value) in custom_env {
            env.insert(key.clone(), value.clone());
        }
    }

    let mut command = Command::new(path).envs(env);
    if let Some(arguments) = command_settings.arguments.as_ref() {
        command = command.args(arguments.clone());
    }

    Ok(command)
}

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

fn command_available(program: &str, args: &[&str]) -> bool {
    let mut command = Command::new(program);
    if !args.is_empty() {
        command = command.args(args.iter().copied());
    }

    matches!(command.output(), Ok(output) if output.status == Some(0))
}

fn find_supported_python() -> Result<PythonCommand> {
    let candidates = [
        PythonCommand {
            program: "python3".to_string(),
            prefix_args: Vec::new(),
        },
        PythonCommand {
            program: "python".to_string(),
            prefix_args: Vec::new(),
        },
        PythonCommand {
            program: "py".to_string(),
            prefix_args: vec!["-3".to_string()],
        },
    ];

    for candidate in candidates {
        if let Some((major, minor)) = python_version(&candidate) {
            if major > PYTHON_MIN_MAJOR || (major == PYTHON_MIN_MAJOR && minor >= PYTHON_MIN_MINOR)
            {
                return Ok(candidate);
            }
        }
    }

    Err("Unable to find `uvx`, `uv`, or a Python 3.10+ interpreter for PAL".to_string())
}

fn python_version(candidate: &PythonCommand) -> Option<(u32, u32)> {
    let output = run_python(candidate, ["-c", PYTHON_VERSION_SCRIPT]).ok()?;
    if output.status != Some(0) {
        return None;
    }

    let version = String::from_utf8(output.stdout).ok()?;
    let mut segments = version.trim().split('.');
    let major = segments.next()?.parse().ok()?;
    let minor = segments.next()?.parse().ok()?;
    Some((major, minor))
}

fn install_pal_with_python(candidate: &PythonCommand) -> Result<()> {
    let output = run_python(
        candidate,
        [
            "-m",
            "pip",
            "install",
            "--user",
            "--upgrade",
            PAL_PACKAGE_NAME,
        ],
    )?;

    if output.status == Some(0) {
        return Ok(());
    }

    Err(format!(
        "Failed to install `{}` via {}: {}",
        PAL_PACKAGE_NAME,
        candidate.program,
        String::from_utf8_lossy(&output.stderr).trim()
    ))
}

fn python_pal_executable(candidate: &PythonCommand) -> Result<String> {
    let output = run_python(candidate, ["-c", PYTHON_SCRIPTS_DIR_SCRIPT])?;
    if output.status != Some(0) {
        return Err(format!(
            "Failed to resolve the PAL executable path via {}: {}",
            candidate.program,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let path = String::from_utf8(output.stdout).map_err(|err| err.to_string())?;
    let path = path.trim().to_string();
    if path.is_empty() {
        return Err("Python reported an empty scripts directory for PAL".to_string());
    }

    Ok(path)
}

fn run_python(
    candidate: &PythonCommand,
    extra_args: impl IntoIterator<Item = &'static str>,
) -> Result<zed::process::Output> {
    let mut command = Command::new(candidate.program.clone());
    if !candidate.prefix_args.is_empty() {
        command = command.args(candidate.prefix_args.clone());
    }

    command.args(extra_args).output()
}

fn push_string_env(env: &mut Vec<(String, String)>, key: &str, value: Option<String>) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        env.push((key.to_string(), value));
    }
}

fn push_value_env<T: ToString>(env: &mut Vec<(String, String)>, key: &str, value: Option<T>) {
    if let Some(value) = value {
        env.push((key.to_string(), value.to_string()));
    }
}

zed::register_extension!(PalMcpServerExtension);
