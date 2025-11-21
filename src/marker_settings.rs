use crate::client::HoneycombClient;
use crate::common::{pretty_print_json, read_json_file, OutputFormat, DEFAULT_TABLE_FORMAT, DEFAULT_PRETTY_FORMAT, CommandContext};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Subcommand)]
pub enum MarkerSettingCommands {
    /// List all marker settings in a dataset
    List {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Environment slug (optional, uses HONEYCOMB_ENVIRONMENT env var if not specified)
        #[arg(short, long, env = "HONEYCOMB_ENVIRONMENT")]
        environment: Option<String>,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_TABLE_FORMAT)]
        format: OutputFormat,
    },
    /// Create a new marker setting
    Create {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Marker setting data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Update a marker setting
    Update {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Marker Setting ID
        #[arg(short, long)]
        id: String,
        /// Marker setting data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Delete a marker setting
    Delete {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Marker Setting ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MarkerSetting {
    pub id: String,
    #[serde(rename = "type")]
    pub setting_type: String,
    pub color: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl MarkerSettingCommands {
    pub async fn execute(&self, client: &HoneycombClient, context: &CommandContext) -> Result<()> {
        match self {
            MarkerSettingCommands::List {
                dataset,
                environment,
                format,
            } => list_marker_settings(client, dataset, environment.as_deref(), format).await,
            MarkerSettingCommands::Create {
                dataset,
                data,
                format,
            } => create_marker_setting(client, dataset, data, format).await,
            MarkerSettingCommands::Update {
                dataset,
                id,
                data,
                format,
            } => update_marker_setting(client, dataset, id, data, format).await,
            MarkerSettingCommands::Delete { dataset, id } => {
                delete_marker_setting(client, dataset, id).await
            }
        }
    }
}

async fn list_marker_settings(
    client: &HoneycombClient,
    dataset: &str,
    environment: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    use crate::common::require_valid_environment;
    use std::collections::HashMap;

    // If environment is provided, validate it exists
    if let Some(env) = environment {
        let team = std::env::var("HONEYCOMB_TEAM").unwrap_or_else(|_| "default".to_string());
        require_valid_environment(client, &team, env).await?;
    }

    let path = format!("/1/marker_settings/{}", dataset);

    // Add environment as query parameter if provided
    let mut query_params = HashMap::new();
    if let Some(env) = environment {
        query_params.insert("environment".to_string(), env.to_string());
    }

    let response = client
        .get(
            &path,
            if query_params.is_empty() {
                None
            } else {
                Some(&query_params)
            },
        )
        .await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty => {
            println!("{}", pretty_print_json(&response)?);
        }
        OutputFormat::Table => {
            if let Value::Array(settings) = response {
                println!("{:<15} {:<30} {:<15} Created", "ID", "Type", "Color");
                println!("{:-<70}", "");

                for setting in settings {
                    if let Ok(ms) = serde_json::from_value::<MarkerSetting>(setting) {
                        println!(
                            "{:<15} {:<30} {:<15} {}",
                            ms.id,
                            ms.setting_type,
                            ms.color,
                            ms.created_at.format("%Y-%m-%d")
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

async fn create_marker_setting(
    client: &HoneycombClient,
    dataset: &str,
    data: &str,
    format: &OutputFormat,
) -> Result<()> {
    let json_data = if std::path::Path::new(data).exists() {
        read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };

    let path = format!("/1/marker_settings/{}", dataset);
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

async fn update_marker_setting(
    client: &HoneycombClient,
    dataset: &str,
    id: &str,
    data: &str,
    format: &OutputFormat,
) -> Result<()> {
    let json_data = if std::path::Path::new(data).exists() {
        read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };

    let path = format!("/1/marker_settings/{}/{}", dataset, id);
    let response = client.put(&path, &json_data).await?;

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

async fn delete_marker_setting(client: &HoneycombClient, dataset: &str, id: &str) -> Result<()> {
    let path = format!("/1/marker_settings/{}/{}", dataset, id);
    client.delete(&path).await?;

    println!(
        "Marker setting '{}' in dataset '{}' deleted successfully",
        id, dataset
    );

    Ok(())
}
