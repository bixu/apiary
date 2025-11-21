use crate::client::HoneycombClient;
use crate::common::{pretty_print_json, read_json_file, OutputFormat};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Subcommand)]
pub enum DatasetCommands {
    /// List all datasets
    List {
        /// Team slug (uses HONEYCOMB_TEAM env var if not specified)
        #[arg(short, long, env = "HONEYCOMB_TEAM")]
        team: Option<String>,
        /// Environment slug (uses HONEYCOMB_ENVIRONMENT env var if not specified)
        #[arg(short, long, env = "HONEYCOMB_ENVIRONMENT")]
        environment: Option<String>,
        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },
    /// Get a specific dataset
    Get {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Create a new dataset
    Create {
        /// Dataset data (JSON file path or inline JSON)
        #[arg(short, long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Update a dataset
    Update {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Dataset data (JSON file path or inline JSON)
        #[arg(short, long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Delete a dataset
    Delete {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Dataset {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_written_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expand_json_depth: Option<u32>,
    pub delete_protected: Option<bool>,
}

impl DatasetCommands {
    pub async fn execute(
        &self,
        client: &HoneycombClient,
        global_team: &Option<String>,
    ) -> Result<()> {
        match self {
            DatasetCommands::List {
                team,
                environment,
                format,
            } => {
                let effective_team = team.as_ref().or(global_team.as_ref())
                    .ok_or_else(|| anyhow::anyhow!("Team is required. Use --team flag or set HONEYCOMB_TEAM environment variable."))?;
                
                // Environment is now optional - if not provided, list all datasets
                list_datasets(client, effective_team, environment.as_deref(), format).await
            }
            DatasetCommands::Get { dataset, format } => get_dataset(client, dataset, format).await,
            DatasetCommands::Create { data, format } => create_dataset(client, data, format).await,
            DatasetCommands::Update {
                dataset,
                data,
                format,
            } => update_dataset(client, dataset, data, format).await,
            DatasetCommands::Delete { dataset } => delete_dataset(client, dataset).await,
        }
    }
}

async fn list_datasets(
    client: &HoneycombClient,
    team: &str,
    environment: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    use crate::common::require_valid_environment;
    use std::collections::HashMap;

    // If environment is provided, validate it exists
    if let Some(env) = environment {
        require_valid_environment(client, team, env).await?;
    }

    // Build query parameters
    let mut query_params = HashMap::new();
    if let Some(env) = environment {
        query_params.insert("environment".to_string(), env.to_string());
    }

    let response = client.get(
        "/1/datasets",
        if query_params.is_empty() {
            None
        } else {
            Some(&query_params)
        },
    ).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty => {
            println!("{}", pretty_print_json(&response)?);
        }
        OutputFormat::Table => {
            if let Value::Array(datasets) = response {
                println!(
                    "{:<30} {:<20} {:<20} Last Written",
                    "Name", "Slug", "Created"
                );
                println!("{:-<80}", "");

                for dataset in datasets {
                    if let Ok(ds) = serde_json::from_value::<Dataset>(dataset) {
                        let last_written = ds
                            .last_written_at
                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                            .unwrap_or_else(|| "Never".to_string());

                        println!(
                            "{:<30} {:<20} {:<20} {}",
                            ds.name,
                            ds.slug,
                            ds.created_at.format("%Y-%m-%d"),
                            last_written
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

async fn get_dataset(client: &HoneycombClient, dataset: &str, format: &OutputFormat) -> Result<()> {
    let path = format!("/1/datasets/{}", dataset);
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

async fn create_dataset(client: &HoneycombClient, data: &str, format: &OutputFormat) -> Result<()> {
    let json_data = if std::path::Path::new(data).exists() {
        read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };

    let response = client.post("/1/datasets", &json_data).await?;

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

async fn update_dataset(
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

    let path = format!("/1/datasets/{}", dataset);
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

async fn delete_dataset(client: &HoneycombClient, dataset: &str) -> Result<()> {
    let path = format!("/1/datasets/{}", dataset);
    client.delete(&path).await?;

    println!("Dataset '{}' deleted successfully", dataset);

    Ok(())
}
