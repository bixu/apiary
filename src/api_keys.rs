use crate::client::HoneycombClient;
use crate::common::{pretty_print_json, read_json_file, OutputFormat};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Subcommand)]
pub enum ApiKeyCommands {
    /// List all API keys in a team
    List {
        /// Team slug
        #[arg(short, long)]
        team: String,
        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },
    /// Get a specific API key
    Get {
        /// Team slug
        #[arg(short, long)]
        team: String,
        /// API Key ID
        #[arg(short, long)]
        id: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Create a new API key
    Create {
        /// Team slug
        #[arg(short, long)]
        team: String,
        /// API key data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Update an API key
    Update {
        /// Team slug
        #[arg(short, long)]
        team: String,
        /// API Key ID
        #[arg(short, long)]
        id: String,
        /// API key data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Delete an API key
    Delete {
        /// Team slug
        #[arg(short, long)]
        team: String,
        /// API Key ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub key_type: String,
    pub environment: Option<ApiKeyEnvironment>,
    pub permissions: Option<ApiKeyPermissions>,
    pub disabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiKeyEnvironment {
    pub id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiKeyPermissions {
    pub events: bool,
    pub markers: bool,
    pub triggers: bool,
    pub boards: bool,
    pub queries: bool,
    pub columns: bool,
    pub slo: bool,
    pub burn_alerts: bool,
    pub datasets: bool,
}

impl ApiKeyCommands {
    pub async fn execute(&self, client: &HoneycombClient) -> Result<()> {
        match self {
            ApiKeyCommands::List { team, format } => list_api_keys(client, team, format).await,
            ApiKeyCommands::Get { team, id, format } => get_api_key(client, team, id, format).await,
            ApiKeyCommands::Create { team, data, format } => {
                create_api_key(client, team, data, format).await
            }
            ApiKeyCommands::Update {
                team,
                id,
                data,
                format,
            } => update_api_key(client, team, id, data, format).await,
            ApiKeyCommands::Delete { team, id } => delete_api_key(client, team, id).await,
        }
    }
}

async fn list_api_keys(client: &HoneycombClient, team: &str, format: &OutputFormat) -> Result<()> {
    let path = format!("/2/teams/{}/api-keys", team);
    let response = client.get(&path, None).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty => {
            println!("{}", pretty_print_json(&response)?);
        }
        OutputFormat::Table => {
            if let Value::Array(api_keys) = response {
                println!(
                    "{:<15} {:<30} {:<15} {:<10} {:<30} {}",
                    "ID", "Name", "Type", "Disabled", "Environment", "Created"
                );
                println!("{:-<110}", "");

                for api_key in api_keys {
                    if let Ok(key) = serde_json::from_value::<ApiKey>(api_key) {
                        let env_name = key
                            .environment
                            .as_ref()
                            .map(|e| e.name.clone())
                            .unwrap_or_else(|| "N/A".to_string());

                        println!(
                            "{:<15} {:<30} {:<15} {:<10} {:<30} {}",
                            key.id,
                            key.name,
                            key.key_type,
                            key.disabled,
                            env_name,
                            key.created_at.format("%Y-%m-%d")
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

async fn get_api_key(
    client: &HoneycombClient,
    team: &str,
    id: &str,
    format: &OutputFormat,
) -> Result<()> {
    let path = format!("/2/teams/{}/api-keys/{}", team, id);
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

async fn create_api_key(
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

    let path = format!("/2/teams/{}/api-keys", team);
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

async fn update_api_key(
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

    let path = format!("/2/teams/{}/api-keys/{}", team, id);
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

async fn delete_api_key(client: &HoneycombClient, team: &str, id: &str) -> Result<()> {
    let path = format!("/2/teams/{}/api-keys/{}", team, id);
    client.delete(&path).await?;

    println!("API Key '{}' in team '{}' deleted successfully", id, team);

    Ok(())
}
