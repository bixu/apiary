use crate::client::HoneycombClient;
use anyhow::Result;
use serde_json::Value;

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
            "Environment '{}' not found in team '{}'. Use 'apiary environments list --team {}' to see available environments.",
            environment,
            team,
            team
        );
    }
    Ok(())
}

pub fn pretty_print_json(value: &serde_json::Value) -> anyhow::Result<String> {
    serde_json::to_string_pretty(value).map_err(Into::into)
}

// Common CLI output formats
#[derive(Debug, Clone)]
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
