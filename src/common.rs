use crate::client::HoneycombClient;
use crate::errors;
use anyhow::Result;
use serde_json::Value;
use std::fmt;

// Constants for consistency
pub const DEFAULT_TABLE_FORMAT: &str = "table";
pub const DEFAULT_PRETTY_FORMAT: &str = "pretty";

// Context for command execution
#[derive(Debug, Clone)]
pub struct CommandContext {
    pub team: Option<String>,
    pub key_material: KeyMaterial,
}

#[derive(Debug, Clone, Default)]
pub struct KeyMaterial {
    pub management: Option<ManagementKeyMaterial>,
    pub configuration: Option<ConfigurationKeyMaterial>,
}

#[derive(Debug, Clone)]
pub struct ManagementKeyMaterial {
    pub id: String,
    pub source: KeySource,
    pub masked: bool,
    pub has_secret: bool,
}

#[derive(Debug, Clone)]
pub struct ConfigurationKeyMaterial {
    pub id: String,
    pub source: KeySource,
}

#[derive(Debug, Clone)]
pub enum KeySource {
    Env(&'static str),
    Flag(&'static str),
}

impl fmt::Display for KeySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeySource::Env(name) => write!(f, "env:{}", name),
            KeySource::Flag(flag) => write!(f, "flag:{}", flag),
        }
    }
}

pub fn mask_identifier(value: &str) -> String {
    if value.len() <= 8 {
        return "*".repeat(value.len());
    }
    let prefix = &value[..4];
    let suffix = &value[value.len() - 4..];
    format!("{}***{}", prefix, suffix)
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
    if !client.has_management_key() {
        // v2 environment lookup requires a management key; skip validation so v1 calls can proceed
        return Ok(true);
    }

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

