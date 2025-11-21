use crate::client::HoneycombClient;
use crate::common::{pretty_print_json, read_json_file, OutputFormat};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};

#[derive(Subcommand)]
pub enum EnvironmentCommands {
    /// List all environments in a team
    List {
        /// Team slug (uses HONEYCOMB_TEAM env var if not specified)
        #[arg(short, long, env = "HONEYCOMB_TEAM")]
        team: Option<String>,
        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },
    /// Get a specific environment
    Get {
        /// Team slug (uses HONEYCOMB_TEAM env var if not specified)
        #[arg(short, long, env = "HONEYCOMB_TEAM")]
        team: Option<String>,
        /// Environment ID
        #[arg(short, long)]
        id: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Create a new environment
    Create {
        /// Team slug (uses HONEYCOMB_TEAM env var if not specified)
        #[arg(short, long, env = "HONEYCOMB_TEAM")]
        team: Option<String>,
        /// Environment data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Update an environment
    Update {
        /// Team slug (uses HONEYCOMB_TEAM env var if not specified)
        #[arg(short, long, env = "HONEYCOMB_TEAM")]
        team: Option<String>,
        /// Environment ID
        #[arg(short, long)]
        id: String,
        /// Environment data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Delete an environment
    Delete {
        /// Team slug (uses HONEYCOMB_TEAM env var if not specified)
        #[arg(short, long, env = "HONEYCOMB_TEAM")]
        team: Option<String>,
        /// Environment ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EnvironmentsResponse {
    pub data: Vec<EnvironmentData>,
    pub links: Option<EnvironmentLinks>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EnvironmentData {
    pub id: String,
    pub attributes: EnvironmentAttributes,
    #[serde(rename = "type")]
    pub data_type: String,
    pub links: Option<EnvironmentSelfLink>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EnvironmentAttributes {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub settings: Option<EnvironmentSettings>,
    pub timestamps: EnvironmentTimestamps,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EnvironmentTimestamps {
    pub created: String,
    pub updated: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EnvironmentLinks {
    pub next: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EnvironmentSelfLink {
    #[serde(rename = "self")]
    pub self_link: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EnvironmentSettings {
    pub delete_protected: Option<bool>,
    pub column_layout: Option<String>,
}

impl EnvironmentCommands {
    pub async fn execute(
        &self,
        client: &HoneycombClient,
        global_team: &Option<String>,
    ) -> Result<()> {
        match self {
            EnvironmentCommands::List { team, format } => {
                let effective_team = team.as_ref().or(global_team.as_ref())
                    .ok_or_else(|| anyhow::anyhow!("Team is required. Use --team flag or set HONEYCOMB_TEAM environment variable."))?;
                list_environments(client, effective_team, format).await
            }
            EnvironmentCommands::Get { team, id, format } => {
                let effective_team = team.as_ref().or(global_team.as_ref())
                    .ok_or_else(|| anyhow::anyhow!("Team is required. Use --team flag or set HONEYCOMB_TEAM environment variable."))?;
                get_environment(client, effective_team, id, format).await
            }
            EnvironmentCommands::Create { team, data, format } => {
                let effective_team = team.as_ref().or(global_team.as_ref())
                    .ok_or_else(|| anyhow::anyhow!("Team is required. Use --team flag or set HONEYCOMB_TEAM environment variable."))?;
                create_environment(client, effective_team, data, format).await
            }
            EnvironmentCommands::Update {
                team,
                id,
                data,
                format,
            } => {
                let effective_team = team.as_ref().or(global_team.as_ref())
                    .ok_or_else(|| anyhow::anyhow!("Team is required. Use --team flag or set HONEYCOMB_TEAM environment variable."))?;
                update_environment(client, effective_team, id, data, format).await
            }
            EnvironmentCommands::Delete { team, id } => {
                let effective_team = team.as_ref().or(global_team.as_ref())
                    .ok_or_else(|| anyhow::anyhow!("Team is required. Use --team flag or set HONEYCOMB_TEAM environment variable."))?;
                delete_environment(client, effective_team, id).await
            }
        }
    }
}

async fn list_environments(
    client: &HoneycombClient,
    team: &str,
    format: &OutputFormat,
) -> Result<()> {
    let path = format!("/2/teams/{}/environments", team);
    let response = client.get(&path, None).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty => {
            println!("{}", pretty_print_json(&response)?);
        }
        OutputFormat::Table => {
            if let Ok(env_response) =
                serde_json::from_value::<EnvironmentsResponse>(response.clone())
            {
                println!(
                    "{:<35} {:<15} {:<25} {:<12} Created",
                    "ID", "Name", "Slug", "Color"
                );
                println!("{:-<95}", "");

                for env_data in env_response.data {
                    let color = env_data
                        .attributes
                        .color
                        .unwrap_or_else(|| "N/A".to_string());
                    println!(
                        "{:<35} {:<15} {:<25} {:<12} {}",
                        env_data.id,
                        env_data.attributes.name,
                        env_data.attributes.slug,
                        color,
                        env_data.attributes.timestamps.created
                    );
                }
            } else {
                println!("{}", pretty_print_json(&response)?);
            }
        }
    }

    Ok(())
}

async fn get_environment(
    client: &HoneycombClient,
    team: &str,
    id: &str,
    format: &OutputFormat,
) -> Result<()> {
    let path = format!("/2/teams/{}/environments/{}", team, id);
    let response = client.get(&path, None).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty | OutputFormat::Table => {
            println!("{}", pretty_print_json(&response)?);
        }
    }

    Ok(())
}

async fn create_environment(
    client: &HoneycombClient,
    team: &str,
    data: &str,
    format: &OutputFormat,
) -> Result<()> {
    let json_data = if std::path::Path::new(data).exists() {
        read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };

    let path = format!("/2/teams/{}/environments", team);
    let response = client.post(&path, &json_data).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty | OutputFormat::Table => {
            println!("{}", pretty_print_json(&response)?);
        }
    }

    Ok(())
}

async fn update_environment(
    client: &HoneycombClient,
    team: &str,
    id: &str,
    data: &str,
    format: &OutputFormat,
) -> Result<()> {
    let json_data = if std::path::Path::new(data).exists() {
        read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };

    let path = format!("/2/teams/{}/environments/{}", team, id);
    let response = client.patch(&path, &json_data).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty | OutputFormat::Table => {
            println!("{}", pretty_print_json(&response)?);
        }
    }

    Ok(())
}

async fn delete_environment(client: &HoneycombClient, team: &str, id: &str) -> Result<()> {
    let path = format!("/2/teams/{}/environments/{}", team, id);
    client.delete(&path).await?;

    println!(
        "Environment '{}' in team '{}' deleted successfully",
        id, team
    );

    Ok(())
}
