use crate::client::HoneycombClient;
use crate::common::{pretty_print_json, read_json_file, OutputFormat};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Subcommand)]
pub enum MarkerCommands {
    /// List all markers in a dataset
    List {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Environment slug (optional, uses HONEYCOMB_ENVIRONMENT env var if not specified)
        #[arg(short, long, env = "HONEYCOMB_ENVIRONMENT")]
        environment: Option<String>,
        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },
    /// Create a new marker
    Create {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Marker data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Update a marker
    Update {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Marker ID
        #[arg(short, long)]
        id: String,
        /// Marker data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Delete a marker
    Delete {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Marker ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Marker {
    pub id: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub url: Option<String>,
    pub color: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl MarkerCommands {
    pub async fn execute(&self, client: &HoneycombClient) -> Result<()> {
        match self {
            MarkerCommands::List { dataset, environment, format } => {
                list_markers(client, dataset, environment.as_deref(), format).await
            },
            MarkerCommands::Create {
                dataset,
                data,
                format,
            } => create_marker(client, dataset, data, format).await,
            MarkerCommands::Update {
                dataset,
                id,
                data,
                format,
            } => update_marker(client, dataset, id, data, format).await,
            MarkerCommands::Delete { dataset, id } => delete_marker(client, dataset, id).await,
        }
    }
}

async fn list_markers(
    client: &HoneycombClient,
    dataset: &str,
    environment: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    use crate::common::require_valid_environment;
    use std::collections::HashMap;

    // If environment is provided, validate it exists
    if let Some(env) = environment {
        let team = std::env::var("HONEYCOMB_TEAM")
            .unwrap_or_else(|_| "default".to_string());
        require_valid_environment(client, &team, env).await?;
    }

    let path = format!("/1/markers/{}", dataset);
    
    // Add environment as query parameter if provided
    let mut query_params = HashMap::new();
    if let Some(env) = environment {
        query_params.insert("environment".to_string(), env.to_string());
    }
    
    let response = client.get(&path, if query_params.is_empty() { None } else { Some(&query_params) }).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty => {
            println!("{}", pretty_print_json(&response)?);
        }
        OutputFormat::Table => {
            if let Value::Array(markers) = response {
                println!(
                    "{:<15} {:<50} {:<20} {:<10} URL",
                    "ID", "Message", "Timestamp", "Color"
                );
                println!("{:-<110}", "");

                for marker in markers {
                    if let Ok(m) = serde_json::from_value::<Marker>(marker) {
                        let url = m.url.unwrap_or_else(|| "N/A".to_string());
                        let color = m.color.unwrap_or_else(|| "N/A".to_string());
                        println!(
                            "{:<15} {:<50} {:<20} {:<10} {}",
                            m.id,
                            m.message,
                            m.timestamp.format("%Y-%m-%d %H:%M"),
                            color,
                            url
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

async fn create_marker(
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

    let path = format!("/1/markers/{}", dataset);
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

async fn update_marker(
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

    let path = format!("/1/markers/{}/{}", dataset, id);
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

async fn delete_marker(client: &HoneycombClient, dataset: &str, id: &str) -> Result<()> {
    let path = format!("/1/markers/{}/{}", dataset, id);
    client.delete(&path).await?;

    println!(
        "Marker '{}' in dataset '{}' deleted successfully",
        id, dataset
    );

    Ok(())
}
