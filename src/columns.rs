use crate::client::HoneycombClient;
use crate::common::{pretty_print_json, read_json_file, OutputFormat, DEFAULT_TABLE_FORMAT, DEFAULT_PRETTY_FORMAT};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Subcommand)]
pub enum ColumnCommands {
    /// List all columns in a dataset
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
    /// Get a specific column
    Get {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Column ID
        #[arg(short, long)]
        id: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Create a new column
    Create {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Column data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Update a column
    Update {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Column ID
        #[arg(short, long)]
        id: String,
        /// Column data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Delete a column
    Delete {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Column ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Column {
    pub id: String,
    pub key_name: String,
    pub hidden: bool,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub column_type: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ColumnCommands {
    pub async fn execute(&self, client: &HoneycombClient) -> Result<()> {
        match self {
            ColumnCommands::List {
                dataset,
                environment,
                format,
            } => list_columns(client, dataset, environment.as_deref(), format).await,

            ColumnCommands::Get {
                dataset,
                id,
                format,
            } => get_column(client, dataset, id, format).await,
            ColumnCommands::Create {
                dataset,
                data,
                format,
            } => create_column(client, dataset, data, format).await,
            ColumnCommands::Update {
                dataset,
                id,
                data,
                format,
            } => update_column(client, dataset, id, data, format).await,
            ColumnCommands::Delete { dataset, id } => delete_column(client, dataset, id).await,
        }
    }
}

async fn list_columns(
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

    let path = format!("/1/columns/{}", dataset);

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
            if let Value::Array(columns) = response {
                println!(
                    "{:<15} {:<30} {:<10} {:<20} Created",
                    "ID", "Key Name", "Hidden", "Type"
                );
                println!("{:-<85}", "");

                for column in columns {
                    if let Ok(col) = serde_json::from_value::<Column>(column) {
                        let col_type = col.column_type.unwrap_or_else(|| "unknown".to_string());
                        println!(
                            "{:<15} {:<30} {:<10} {:<20} {}",
                            col.id,
                            col.key_name,
                            col.hidden,
                            col_type,
                            col.created_at.format("%Y-%m-%d")
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

async fn get_column(
    client: &HoneycombClient,
    dataset: &str,
    id: &str,
    format: &OutputFormat,
) -> Result<()> {
    let path = format!("/1/columns/{}/{}", dataset, id);
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

async fn create_column(
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

    let path = format!("/1/columns/{}", dataset);
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

async fn update_column(
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

    let path = format!("/1/columns/{}/{}", dataset, id);
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

async fn delete_column(client: &HoneycombClient, dataset: &str, id: &str) -> Result<()> {
    let path = format!("/1/columns/{}/{}", dataset, id);
    client.delete(&path).await?;

    println!(
        "Column '{}' in dataset '{}' deleted successfully",
        id, dataset
    );

    Ok(())
}
