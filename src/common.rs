use crate::client::HoneycombClient;
use crate::errors;
use anyhow::Result;
use serde_json::Value;

// Constants for consistency
pub const DEFAULT_TABLE_FORMAT: &str = "table";
pub const DEFAULT_PRETTY_FORMAT: &str = "pretty";
#[allow(dead_code)]
pub const HONEYCOMB_TEAM_ENV: &str = "HONEYCOMB_TEAM";
#[allow(dead_code)]
pub const HONEYCOMB_ENVIRONMENT_ENV: &str = "HONEYCOMB_ENVIRONMENT";

// Context for command execution
#[derive(Debug, Clone)]
pub struct CommandContext {
    pub team: Option<String>,
    #[allow(dead_code)]
    pub global_format: Option<OutputFormat>,
    #[allow(dead_code)]
    pub verbose: bool,
}

// Common utility functions
pub fn read_json_file(path: &str) -> anyhow::Result<serde_json::Value> {
    let content = std::fs::read_to_string(path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    Ok(json)
}

// Environment validation function
pub async fn validate_environment(
    client: &HoneycombClient,
    team: &str,
    environment: &str,
) -> Result<bool> {
    let path = format!("/2/teams/{}/environments", team);
    let response = client.get(&path, None).await?;

    if let Value::Object(obj) = response {
        if let Some(Value::Array(envs)) = obj.get("data") {
            for env in envs {
                if let Value::Object(env_obj) = env {
                    if let Some(Value::Object(attrs)) = env_obj.get("attributes") {
                        if let Some(Value::String(slug)) = attrs.get("slug") {
                            if slug == environment {
                                return Ok(true);
                            }
                        }
                        if let Some(Value::String(name)) = attrs.get("name") {
                            if name == environment {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(false)
}

// Validate and throw error if environment doesn't exist
pub async fn require_valid_environment(
    client: &HoneycombClient,
    team: &str,
    environment: &str,
) -> Result<()> {
    if !validate_environment(client, team, environment).await? {
        anyhow::bail!(
            "{}",
            errors::messages::environment_not_found(environment, team)
        );
    }
    Ok(())
}

pub fn pretty_print_json(value: &serde_json::Value) -> anyhow::Result<String> {
    serde_json::to_string_pretty(value).map_err(Into::into)
}

// Common CLI output formats
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Json,
    Pretty,
    Table,
}

impl std::str::FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "pretty" => Ok(OutputFormat::Pretty),
            "table" => Ok(OutputFormat::Table),
            _ => anyhow::bail!("Invalid output format. Use: json, pretty, or table"),
        }
    }
}

// Macro for standard team parameter
#[macro_export]
macro_rules! team_param {
    () => {
        /// Team slug (uses HONEYCOMB_TEAM env var if not specified)
        #[arg(short, long, env = $crate::common::HONEYCOMB_TEAM_ENV)]
        team: Option<String>,
    };
}

// Macro for standard environment parameter
#[macro_export]
macro_rules! environment_param {
    () => {
        /// Environment slug (uses HONEYCOMB_ENVIRONMENT env var if not specified)
        #[arg(short, long, env = $crate::common::HONEYCOMB_ENVIRONMENT_ENV)]
        environment: Option<String>,
    };
}

// Macro for standard format parameter
#[macro_export]
macro_rules! format_param {
    ($default:expr) => {
        /// Output format
        #[arg(short, long, default_value = $default)]
        format: OutputFormat,
    };
}

// Helper function to resolve team parameter
#[allow(dead_code)]
pub fn resolve_team(local_team: &Option<String>, context: &CommandContext) -> Result<String> {
    local_team
        .as_ref()
        .or(context.team.as_ref())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!(errors::messages::TEAM_REQUIRED))
}
